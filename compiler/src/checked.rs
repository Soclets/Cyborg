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
    Value { value: Expression, shape: Shape },
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

/// What a failed check prints. The message is rendered while building, in the one error layout (standard 11.1),
/// with the values that broke it, if any, printed into its `when:` line while running.
#[derive(Clone, Debug)]
pub struct CheckFailure {
    /// The text before the `when:` values, or the whole message.
    pub before: String,
    /// `when:` parts: text, then which operand's value, then `after`.
    pub when: Vec<(String, usize)>,
    pub after: String,
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

#[derive(Clone, Debug)]
pub enum Argument {
    Value(Expression),
    /// The address of a local, for a lent or changed parameter.
    Address(LocalId),
}
