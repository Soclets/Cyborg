//! What the checker hands to the runner: every name resolved, every type settled, and every hidden
//! check written out as a node.

use crate::ast::{CompareOperator, Narrowing};
use crate::report::Span;
use crate::types::{FloatKind, FloatRules, IntegerKind, Value};

/// A variable and the fields and items to walk from it.
#[derive(Clone, Debug)]
pub struct Place {
    pub slot: usize,
    pub path: Vec<Step>,
}

#[derive(Clone, Debug)]
pub enum Step {
    Field(usize),
    Item(Box<Position>),
}

/// `$xs[pos: 1]` or `$xs[offset: 0]`, worked out while running, since it can be any number.
#[derive(Clone, Debug)]
pub struct Position {
    pub value: Node,
    pub counts_from_zero: bool,
    /// How `what:` says the collection, as in `$xs`.
    pub shown: String,
    pub span: Span,
}

/// A piece of a format print: text, or a hole with the value it takes, counting from 0. (D194)
#[derive(Clone, Debug)]
pub enum PrintPiece {
    Text(String),
    Hole {
        value: usize,
        shapes: crate::ast::Shapes,
        span: Span,
    },
}

/// Where a returned borrow points: a place, or what a call hands back. (D170)
#[derive(Clone, Debug)]
pub enum Alias {
    Place(Place),
    Call(Node),
}

/// Stands for an item in a path that's compared while checking, since which one isn't known.
pub const ANY_ITEM: usize = usize::MAX;

impl Place {
    /// The path as the checker compares it, where every item looks alike.
    pub fn keys(&self) -> Vec<usize> {
        self.path
            .iter()
            .map(|step| match step {
                Step::Field(index) => *index,
                Step::Item(_) => ANY_ITEM,
            })
            .collect()
    }
}

/// Whether two paths from one variable surely touch. Two different fields don't, and neither is
/// sure for two items, since which ones they are may only be known while running.
/// Whether two paths can reach the same value: two items of one list may be the same one, so
/// they're held together. (D173)
pub fn paths_may_meet(first: &[usize], second: &[usize]) -> bool {
    first
        .iter()
        .zip(second)
        .all(|(one, other)| *one == *other || *one == ANY_ITEM || *other == ANY_ITEM)
}

pub fn paths_overlap(first: &[usize], second: &[usize]) -> bool {
    first
        .iter()
        .zip(second)
        .all(|(one, other)| *one == *other && *one != ANY_ITEM)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Arithmetic {
    Add,
    Subtract,
    Multiply,
    /// Wraps in every mode. (D295)
    Wrap(crate::ast::WordOperator),
    /// Sticks at these bounds: the type's written ones, or the hardware's. (D294)
    Saturate(crate::ast::WordOperator, i128, i128),
    Divide(Narrowing),
    Remainder(Narrowing),
    BitAnd,
    BitOr,
    BitXor,
    ShiftLeft,
    ShiftRight,
}

impl Arithmetic {
    /// The plain operator whose unit rules this follows: `plus.wrap` adds units as `+` does.
    pub fn unit_rule(self) -> Arithmetic {
        use crate::ast::WordOperator;
        match self {
            Arithmetic::Wrap(word) | Arithmetic::Saturate(word, ..) => match word {
                WordOperator::Plus => Arithmetic::Add,
                WordOperator::Minus => Arithmetic::Subtract,
                WordOperator::Times => Arithmetic::Multiply,
            },
            other => other,
        }
    }
}

/// How a type parameter's type does each operation: by name, as `<` or `div.floor`.
pub type Table = Vec<(String, Implementation)>;

/// How one type does one operation for a generic. (D235, D236)
#[derive(Clone, Debug)]
pub enum Implementation {
    /// A built-in number's own arithmetic.
    Arithmetic(Arithmetic, Number),
    /// A built-in comparison, `==` or `<`.
    Compare(crate::ast::CompareOperator),
    /// A type's own `op`, by its function.
    Function(usize),
    /// What the generic calling this one was told for its own type parameter.
    Forward { parameter: usize, operator: String },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Number {
    Integer(IntegerKind),
    Float(FloatKind, FloatRules),
}

#[derive(Clone, Debug)]
pub struct Node {
    pub span: Span,
    pub kind: NodeKind,
}

#[derive(Clone, Debug)]
pub enum Argument {
    /// A fresh or copied value, which the callee owns.
    Value(Node),
    /// `Lend` or `Lend.mut`: the callee works on the caller's own value.
    Alias(Place),
}

#[derive(Clone, Debug)]
pub enum NodeKind {
    Constant(Value),
    Read(Place),
    Global(usize),
    Field(Box<Node>, usize),
    /// An item of something with no variable to walk from, such as a constant or a call's result.
    Item(Box<Node>, Box<Position>),
    Struct(Vec<(usize, Node)>),
    Case(usize, Vec<(usize, Node)>),
    Items(Vec<Node>),
    All(Box<Node>, usize),
    Arithmetic {
        operator: Arithmetic,
        number: Number,
        left: Box<Node>,
        right: Box<Node>,
    },
    BitNot(IntegerKind, Box<Node>),
    And(Box<Node>, Box<Node>),
    Or(Box<Node>, Box<Node>),
    AndBoth(Box<Node>, Box<Node>),
    OrBoth(Box<Node>, Box<Node>),
    Negate(Number, Box<Node>),
    Not(Box<Node>),
    Compare {
        first: Box<Node>,
        rest: Vec<(CompareOperator, Node)>,
    },
    /// `arguments` are in the order written, and each says which parameter it fills. A
    /// generic's call also carries, for each type parameter, how the type that fills it does
    /// each operation its requirement lists. (D235)
    Call {
        function: usize,
        arguments: Vec<(usize, Argument)>,
        tables: Vec<Table>,
    },
    /// An operation on a type parameter's values, done by what the call's table says. (D235)
    Generic {
        parameter: usize,
        operator: String,
        left: Box<Node>,
        right: Box<Node>,
    },
    /// `std::io::fmt_print`, with the writer it goes to, its text in pieces and the values its
    /// holes take. (D194, D272)
    FormatPrint {
        writer: Box<Node>,
        pieces: Vec<PrintPiece>,
        values: Vec<Node>,
        stored: bool,
    },
    /// `os::stdout.buffered(size: 1024 bytes)` or `os::stderr.unbuffered()`: a terminal writer,
    /// buffered when it has a size. (D273)
    Writer {
        stderr: bool,
        size: Option<usize>,
    },
    /// `$out.%%flush()`, or `os::done(Give $out)`, which is the same flush with the duty ended
    /// when checking. (D273)
    Flush {
        writer: Box<Node>,
    },
    Append {
        receiver: Place,
        shown: String,
        item: Box<Node>,
        max: Option<i128>,
        /// The most its `len.type` can count, which `inf` can run into. (D140)
        count_limit: i128,
    },
    Length(Box<Node>),
    IntegerToStr(Box<Node>),
    /// A failure with nothing handling it stops the program. (D96)
    Crash(Box<Node>),
    Fallback {
        value: Box<Node>,
        fallback: Box<Node>,
    },
    Handle {
        value: Box<Node>,
        arms: Vec<FailureArm>,
    },
    /// `std::math::sin` and the rest, correctly rounded. (D256)
    Maths {
        function: crate::maths::Function,
        value: Box<Node>,
    },
    /// A unit's decibels of a reference, or back: `reference x 10^(value / 10)` when
    /// `to_amount`, and `10 x log10(value / reference)` when not. (D251, D255)
    Decibels {
        value: Box<Node>,
        number: Number,
        reference: f64,
        to_amount: bool,
    },
    /// The same for a float. (D158)
    FitFloat {
        value: Box<Node>,
        low: f64,
        high: f64,
        target: String,
        shown: Option<String>,
    },
    /// `$x as str`, in a float's shortest text. (D161)
    FloatToStr(Number, Box<Node>),

    /// `$reading as f64`, which fails on NaN or infinity. (D159)
    FloatFromIeee {
        value: Box<Node>,
        target: String,
    },
    /// A plain `=`, `change` or argument checks a bound the compiler can't see fits. (D20)
    Fit {
        value: Box<Node>,
        low: i128,
        high: i128,
        target: String,
        shown: Option<String>,
    },
    /// The name is `void` here too, so `?? void` has something to give back.
    Void,
}

/// A range's numbers, each worked out once, before the loop starts.
#[derive(Clone, Debug)]
pub struct RangeCode {
    pub from: Node,
    pub end: Node,
    pub included: bool,
    pub step: Option<Node>,
}

#[derive(Clone, Debug)]
pub struct Instruction {
    pub span: Span,
    pub kind: InstructionKind,
}

#[derive(Clone, Debug)]
pub enum InstructionKind {
    Let {
        slot: usize,
        value: Node,
    },
    /// A stored borrow names its source instead of holding a value of its own.
    LetAlias {
        slot: usize,
        place: Place,
    },
    Set {
        place: Place,
        value: Node,
    },
    If {
        links: Vec<(Test, Vec<Instruction>)>,
        otherwise: Vec<Instruction>,
    },
    Match {
        subject: Node,
        arms: Vec<MatchArm>,
    },
    For {
        loop_id: usize,
        slot: usize,
        collection: Node,
        body: Vec<Instruction>,
    },
    /// `for $r in Lend.mut $xs`, where `$r` is each item in place. (D180)
    ForInPlace {
        loop_id: usize,
        slot: usize,
        place: Place,
        body: Vec<Instruction>,
    },
    ForRange {
        loop_id: usize,
        slot: usize,
        range: Box<RangeCode>,
        body: Vec<Instruction>,
    },
    While {
        loop_id: usize,
        condition: Node,
        body: Vec<Instruction>,
    },
    Forever {
        loop_id: usize,
        body: Vec<Instruction>,
    },
    Break(usize),
    Instead(Node),
    Return(Node),
    /// `return Lend $a;`, handing back a borrow rather than a value. (D170)
    ReturnBorrow(Alias),
    /// `let $w: Lend from <$x, $y> reading = warmer(Lend $x, Lend $y);`, a borrow a call hands
    /// back. (D170)
    LetCallAlias {
        slot: usize,
        call: Node,
    },
    Fail {
        name: String,
        shown: String,
        data: Vec<Node>,
    },
    Evaluate(Node),
    Block(Vec<Instruction>),
}

/// One `case` in a `match`. `slot` is where `case status.failed $f:` keeps its name.
#[derive(Clone, Debug)]
pub struct MatchArm {
    pub case: usize,
    pub slot: Option<usize>,
    pub body: Vec<Instruction>,
}

/// One `case` in an `otherwise`. `slot` is where `case negative $f:` keeps its name.
#[derive(Clone, Debug)]
pub struct FailureArm {
    pub failure: String,
    pub slot: Option<usize>,
    pub body: Vec<Instruction>,
}

/// What decides whether one block of an `if` runs.
#[derive(Clone, Debug)]
pub enum Test {
    Condition(Node),
    Range {
        subject: Node,
        low: i128,
        high: i128,
    },
    FloatRange {
        subject: Node,
        low: f64,
        high: f64,
    },
    Case {
        subject: Node,
        case: usize,
    },
}

#[derive(Clone, Debug)]
pub struct Contract {
    pub condition: Node,
    /// Values for `when:`, read only once the contract has come out false.
    pub shown: Vec<(String, Node)>,
}

#[derive(Clone, Debug)]
pub struct Function {
    /// Each parameter's slot is its position, so the other variables come after them.
    pub slot_count: usize,
    pub result_slot: usize,
    pub requires: Vec<Contract>,
    pub ensures: Vec<Contract>,
    pub body: Vec<Instruction>,
}

#[derive(Clone, Debug)]
pub struct Code {
    pub main: usize,
    pub functions: Vec<Function>,
    /// In source order, so one can read an earlier one.
    pub globals: Vec<Node>,
    /// Everything `bake` and local `const` need worked out while building.
    pub compile_time: Vec<Node>,
}
