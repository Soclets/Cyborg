//! What the checker hands to the runner: every name resolved, every type settled, and every hidden
//! check written out as a node.

use crate::ast::{CompareOperator, Narrowing};
use crate::report::Span;
use crate::types::{FloatKind, IntegerKind, Value};

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
    PlusWrap,
    Divide(Narrowing),
    Remainder(Narrowing),
    BitAnd,
    BitOr,
    BitXor,
    ShiftLeft,
    ShiftRight,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Number {
    Integer(IntegerKind),
    Float(FloatKind),
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
    /// `arguments` are in the order written, and each says which parameter it fills.
    Call {
        function: usize,
        arguments: Vec<(usize, Argument)>,
    },
    Println(Vec<Node>),
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
