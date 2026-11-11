//! The checked program, which code generation reads.
//!
//! The checker has already refused everything that's wrong, so this holds only what runs: plain machine types,
//! values and the runtime checks the build mode decides about (standard 10.3).

/// A machine type.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum MachineType {
    Integer {
        bits: u32,
        signed: bool,
    },
    Float {
        bits: u32,
    },
    Bool,
    Void,
    /// A string: its address and its length in bytes.
    Text,
    /// A writer from `os`, buffered or not (D272, D273). Its runtime record is the same for both.
    Writer,
    /// The address of a place, which a function hands back as a borrow (D170).
    Address,
    /// A fixed number of items, held in place (D12).
    Array {
        item: Box<MachineType>,
        len: u64,
    },
    /// Items on the heap: their address, how many there are, and how many fit before it grows (D11).
    List(Box<MachineType>),
    /// A struct, by its number in the program.
    Struct(usize),
    /// A tagged union: a tag saying which case, then room for the largest case.
    Union(usize),
}

impl MachineType {
    pub const I32: MachineType = MachineType::Integer { bits: 32, signed: true };
    pub const I64: MachineType = MachineType::Integer { bits: 64, signed: true };
    pub const U64: MachineType = MachineType::Integer { bits: 64, signed: false };
}

pub type LocalId = usize;
pub type FunctionId = usize;
pub type LoopId = usize;

#[derive(Clone, Debug, Default)]
pub struct Program {
    pub functions: Vec<Function>,
    /// Each struct's fields.
    pub structs: Vec<Vec<MachineType>>,
    /// Each union's cases, and each case's fields.
    pub unions: Vec<Vec<Vec<MachineType>>>,
    /// Where each failure was raised, as the `because:` part its crash message ends with.
    pub sites: Vec<String>,
    /// `fn.main`.
    pub entry: FunctionId,
}

#[derive(Clone, Debug)]
pub struct Function {
    /// The symbol, unique in the program.
    pub symbol: String,
    pub parameters: Vec<Parameter>,
    pub returns: MachineType,
    /// Every local's type, parameters first.
    pub locals: Vec<MachineType>,
    pub body: Block,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Passing {
    /// By value: a plain number or `bool` (D184).
    Value,
    /// By address, read-only or to change.
    Address,
}

#[derive(Clone, Debug)]
pub struct Parameter {
    pub local: LocalId,
    pub passing: Passing,
}

pub type Block = Vec<Statement>;

#[derive(Clone, Debug)]
pub enum Statement {
    /// Gives a local its first value.
    Let {
        local: LocalId,
        value: Expression,
    },
    /// Changes a local.
    Set {
        local: LocalId,
        value: Expression,
    },
    /// A check whose crash message ends with where the function was called from, from the sites table (D29).
    CheckAtSite {
        condition: Expression,
        failure: CheckFailure,
        operands: Vec<Expression>,
        site: Expression,
    },
    /// `instead value;` in an `otherwise:` arm: the value used in place of the failed call (D139).
    Instead {
        local: LocalId,
        value: Expression,
    },
    /// Changes one field, maybe deep inside a local: `change $r.celsius = 22;` (D125).
    SetField {
        local: LocalId,
        path: Vec<usize>,
        value: Expression,
    },
    Evaluate(Expression),
    If {
        condition: Expression,
        then: Block,
        otherwise: Block,
    },
    /// Runs its body until a `Break` names it.
    Loop {
        id: LoopId,
        body: Block,
        /// Where the loop starts, for an error about compile-time work that doesn't finish.
        opener: crate::source::Span,
    },
    Break(LoopId),
    Return(Option<Expression>),
    /// Opens a writer in a local: `os::stdout.buffered(size: 1024 bytes)`.
    OpenWriter {
        local: LocalId,
        stream: Stream,
        capacity: Option<Expression>,
    },
    /// A format print, already split into pieces while building (D197).
    Print {
        writer: LocalId,
        pieces: Vec<Piece>,
    },
    Flush {
        writer: LocalId,
    },
    /// `os::done(Give $out)`: flushes for the last time and frees the buffer (D273).
    Done {
        writer: LocalId,
    },
    /// Changes what's at a place: `change $xs[pos: 3] = 10;`. The place is worked out first, then the value (D133).
    Store {
        place: Place,
        value: Expression,
    },
    /// Adds an item at the end of a list, checking it has room for one more (D24).
    Push {
        place: Place,
        value: Expression,
        room: Option<(u64, Box<CheckFailure>)>,
    },
    /// A stored borrow made from a call that hands one back: from here on, `local` is the place at that address
    /// (D170).
    AliasValue {
        local: LocalId,
        value: Expression,
    },
    /// A stored borrow: from here on, `local` is the place it borrows (D2, D169).
    Alias {
        local: LocalId,
        target: LocalId,
        path: Vec<usize>,
    },
    /// What the statement after it does to ownership, for the checks over the control-flow IR. It runs nothing
    /// (D285).
    Flow(Vec<crate::flow::Event>),
    /// Stops the program with its failure's message unless the condition holds. The operands are the values its
    /// `when:` line prints.
    Check {
        condition: Expression,
        failure: CheckFailure,
        operands: Vec<Expression>,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Stream {
    Out,
    Error,
}

impl Stream {
    #[must_use]
    pub fn descriptor(self) -> i32 {
        match self {
            Stream::Out => 1,
            Stream::Error => 2,
        }
    }
}

/// A piece of a format print.
#[derive(Clone, Debug)]
pub enum Piece {
    Text(String),
    /// A value with its shape. A float's print can fail in ways that aren't decided: a value exactly halfway for
    /// `places:`, `form: science` with an exponent of 0 or below, and NaN with shapes (standard 12.2).
    Value {
        value: Expression,
        shape: Shape,
        failures: Option<Box<FloatPrintFailures>>,
    },
}

/// What a float's print says when it reaches something undecided.
#[derive(Clone, Debug)]
pub struct FloatPrintFailures {
    pub halfway: CheckFailure,
    pub science: CheckFailure,
    pub nan: CheckFailure,
}

/// How a hole prints its value (D194).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Shape {
    pub width: Option<u64>,
    /// What a width counts (D194).
    pub width_unit: WidthUnit,
    pub align: Option<Align>,
    pub zeros: bool,
    pub sign: bool,
    pub base: Base,
    pub places: Option<u32>,
    pub science: bool,
    pub stored: bool,
}

/// What a width in a print counts (D194).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum WidthUnit {
    /// Terminal columns, where East Asian wide characters take two.
    #[default]
    Columns,
    /// Columns, where characters of ambiguous width take two as well, as East Asian terminals show them.
    ColumnsEastAsian,
    Graphemes,
    Bytes,
}

impl Shape {
    /// How wide a piece of text is, in the width's unit.
    #[must_use]
    pub fn columns_of(&self, text: &str) -> u64 {
        use unicode_segmentation::UnicodeSegmentation;
        use unicode_width::UnicodeWidthStr;
        let count = match self.width_unit {
            WidthUnit::Columns => text.width(),
            WidthUnit::ColumnsEastAsian => text.width_cjk(),
            WidthUnit::Graphemes => text.graphemes(true).count(),
            WidthUnit::Bytes => text.len(),
        };
        count as u64
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Align {
    Left,
    Right,
    Center,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Base {
    #[default]
    Decimal,
    Binary,
    Octal,
    Hex,
    HexUpper,
}

/// What happens when a call fails (standard 8).
#[derive(Clone, Debug)]
pub enum OnFailure {
    /// Nothing handles it, so the program stops (D96).
    Crash(Vec<Unhandled>),
    /// `?? void` on a call that gives nothing back.
    Ignore,
    /// `?? value`.
    Fallback(Expression),
    /// `?`: the caller's result, with the callee's failure cases moved to the caller's (D47).
    PassUp { mapping: Vec<(usize, usize)>, structure: usize, union: usize, value_type: MachineType },
    /// `otherwise:` arms, by failure case, giving their value with `instead` into a local.
    Arms { arms: Vec<(usize, Block)>, result: LocalId, union_local: LocalId },
}

/// The crash message for one failure nothing handles.
#[derive(Clone, Debug)]
pub struct Unhandled {
    pub case: usize,
    pub failure: CheckFailure,
    /// Whether it carries one value, which its `when:` line prints.
    pub carried: bool,
}

/// What a failed check prints. The message is rendered while building, in the one error layout (standard 11.1),
/// with the values that broke it, if any, printed into its `when:` line while running.
#[derive(Clone, Debug)]
pub struct CheckFailure {
    /// The text before the `when:` values, or the whole message.
    pub before: String,
    /// `when:` parts: text, then which operand's value, then `after`.
    pub when: Vec<(String, usize)>,
    pub after: String,
    /// The error itself, and its `when:` line's parts, so a check that fails while baking can be refused instead
    /// (D91).
    pub source: Option<Box<crate::diagnostic::Diagnostic>>,
    pub when_template: Vec<WhenPart>,
}

/// A part of a `when:` line: text, or an operand's value.
#[derive(Clone, Debug)]
pub enum WhenPart {
    Text(String),
    Operand(usize),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Arithmetic {
    Add,
    Subtract,
    Multiply,
    /// `div.trunc`.
    DivideTruncate,
    /// `div.floor`.
    DivideFloor,
    /// `div.exact`, whose inexact case isn't decided (standard 12.3).
    DivideExact,
    RemainderTruncate,
    RemainderFloor,
    WrappingAdd,
    BitAnd,
    BitOr,
    BitXor,
    ShiftLeft,
    ShiftRight,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FloatArithmetic {
    Add,
    Subtract,
    Multiply,
    Divide,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Comparison {
    Equal,
    NotEqual,
    Below,
    Above,
    AtLeast,
    AtMost,
}

#[derive(Clone, Debug)]
pub struct Expression {
    pub kind: ExpressionKind,
    pub machine_type: MachineType,
}

#[derive(Clone, Debug)]
pub enum ExpressionKind {
    Integer(i128),
    Float(f64),
    Bool(bool),
    /// A string literal's text.
    Text(String),
    Local(LocalId),
    /// Arithmetic, with the check that stops overflow and division by zero, where the mode keeps one, and any more
    /// checks that come after it, like shifting too far.
    Arithmetic {
        operator: Arithmetic,
        left: Box<Expression>,
        right: Box<Expression>,
        failure: Option<Box<CheckFailure>>,
        more: Vec<CheckFailure>,
    },
    Negate {
        operand: Box<Expression>,
        failure: Option<Box<CheckFailure>>,
    },
    Compare {
        operator: Comparison,
        left: Box<Expression>,
        right: Box<Expression>,
    },
    Not(Box<Expression>),
    BitNot(Box<Expression>),
    /// `and` and `or` stop early (D124).
    And(Box<Expression>, Box<Expression>),
    Or(Box<Expression>, Box<Expression>),
    /// Widens or narrows a number between integer types.
    Convert(Box<Expression>),
    Call {
        function: FunctionId,
        arguments: Vec<Argument>,
    },
    /// Float arithmetic, with the checks that a `.stable` or `.fast` result isn't infinity or NaN (D158).
    FloatArithmetic {
        operator: FloatArithmetic,
        left: Box<Expression>,
        right: Box<Expression>,
        infinity: Option<Box<CheckFailure>>,
        nan: Option<Box<CheckFailure>>,
    },
    /// A number's text, from `as str` (D122, D161).
    TextOf(Box<Expression>),
    /// An `.ieee` float crossing into `.stable` or `.fast` with `as`, which fails on NaN and infinity (D159).
    FromIeee {
        value: Box<Expression>,
        nan: Box<CheckFailure>,
        infinity: Box<CheckFailure>,
    },
    /// A float checked to be within a bound while running (D158).
    FitFloat {
        value: Box<Expression>,
        min: f64,
        max: f64,
        failure: Box<CheckFailure>,
    },
    /// A value with every bit 0, for a result's value when it failed.
    Zero,
    /// A call to a failing function, and what happens when it fails. Its value is the call's value.
    Handle {
        call: Box<Expression>,
        /// The union of the callee's failures, inside its result.
        union: usize,
        on_failure: Box<OnFailure>,
    },
    /// A struct built from its fields' values, in their declared order.
    Build(Vec<Expression>),
    /// The value at a place, through fields and items, each item checked (D133).
    Load(Place),
    /// The address of a place, for a borrow of an item (D133, D180).
    PlaceAddress(Place),
    /// A list built from its items (D102).
    ListOf(Vec<Expression>),
    /// An array of one value, `all 0` (D102).
    Fill {
        value: Box<Expression>,
        len: u64,
    },
    /// How many items a list holds, as a `u64`.
    ListLen(Box<Expression>),
    /// The address of a local, or a field inside it, handed back as a borrow (D170).
    AddressOf {
        local: LocalId,
        path: Vec<usize>,
    },
    /// One field of a struct value.
    Field {
        base: Box<Expression>,
        index: usize,
    },
    /// A union's case, built from its values.
    BuildCase {
        case: usize,
        fields: Vec<Expression>,
    },
    /// Which case a union holds.
    Tag(Box<Expression>),
    /// One field of the case a union is known to hold.
    CaseField {
        base: Box<Expression>,
        case: usize,
        index: usize,
    },
    /// A value checked to be within a bound while running (D20).
    Fit {
        value: Box<Expression>,
        min: i128,
        max: i128,
        failure: Box<CheckFailure>,
    },
    /// Puts a value in a local, then gives `body`, so a value used twice is evaluated once.
    Bind {
        local: LocalId,
        value: Box<Expression>,
        body: Box<Expression>,
    },
}

/// A place in memory: a local, then fields and items inside it (D125, D133).
#[derive(Clone, Debug)]
pub struct Place {
    pub local: LocalId,
    pub steps: Vec<Step>,
}

#[derive(Clone, Debug)]
pub enum Step {
    Field(usize),
    /// An item at a position as written, counting from 1 for `pos:` or 0 for `offset.items:`, with its bounds check,
    /// whose `when:` prints the position and the count.
    Item {
        position: Box<Expression>,
        from_one: bool,
        check: Option<Box<CheckFailure>>,
    },
}

#[derive(Clone, Debug)]
pub enum Argument {
    Value(Expression),
    /// The address of a local, for a lent or changed parameter.
    Address(LocalId),
    /// The address of a field inside a local, for a lent field: `Lend $o.b`.
    FieldAddress {
        local: LocalId,
        path: Vec<usize>,
    },
    /// A fresh value lent for the call: it's put in a temporary, whose address is passed.
    Temporary(Expression),
    /// The address of a place through items, lent: `Lend.mut $a[pos: 1]` (D133).
    Place(Place),
}
