//! The checked program, which code generation reads.
//!
//! The checker has already refused everything that's wrong, so this holds only what runs: plain machine types,
//! values and the runtime checks the build mode decides about (standard 10.3).

/// A machine type.
#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
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
    Struct(#[serde(with = "struct_number")] usize),
    /// A tagged union: a tag saying which case, then room for the largest case.
    Union(#[serde(with = "union_number")] usize),
}

impl MachineType {
    pub const I32: MachineType = MachineType::Integer { bits: 32, signed: true };
    pub const I64: MachineType = MachineType::Integer { bits: 64, signed: true };
    pub const U8: MachineType = MachineType::Integer { bits: 8, signed: false };
    pub const U64: MachineType = MachineType::Integer { bits: 64, signed: false };
}

pub type LocalId = usize;
pub type FunctionId = usize;
pub type LoopId = usize;

#[derive(Clone, Debug, Default)]
pub struct Program {
    pub functions: Vec<Function>,
    /// Each struct's fields, in the order they're laid out: written, or sorted for `.compact` (D319, D388).
    pub structs: Vec<Vec<MachineType>>,
    /// Whether each struct is `.packed`, with no padding between its fields (D319).
    pub packed: Vec<bool>,
    /// Each union's cases, and each case's fields.
    pub unions: Vec<Vec<Vec<MachineType>>>,
    /// `fn.main`.
    pub entry: FunctionId,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Function {
    /// The symbol, unique in the program.
    pub symbol: String,
    pub parameters: Vec<Parameter>,
    pub returns: MachineType,
    /// Every local's type, parameters first.
    pub locals: Vec<MachineType>,
    pub body: Block,
    /// Where this function raises a failure or calls one with `requires`, as the `because:` part a crash message
    /// ends with (D29, D96). A site travels as a pointer to its text, so each function keeps its own.
    pub sites: Vec<Site>,
    /// A generic's own body, which is only checked: each call builds a copy with real types (D235).
    #[serde(default)]
    pub template: bool,
    /// Each local's bounds, if it's a bounded integer, which hold wherever it's read (D312).
    #[serde(default)]
    pub local_bounds: Vec<Option<(i128, i128)>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Passing {
    /// By value: a plain number or `bool` (D184).
    Value,
    /// By address, read-only or to change.
    Address,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Parameter {
    pub local: LocalId,
    pub passing: Passing,
}

pub type Block = Vec<Statement>;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum Statement {
    /// Starts a call as a task on one of an OS pool's threads, keeping its record in `task` (D343, D345).
    TaskStart {
        task: LocalId,
        pool: Expression,
        call: Expression,
    },
    /// Waits for the task whose record is in `task`, puts what it gave back in `result`, and frees the record (D343).
    TaskJoin {
        task: LocalId,
        result: LocalId,
    },
    /// Ends an OS pool, whose tasks are all joined (D392).
    PoolDone(Expression),
    /// Takes every lock whose word's address is in an array local, in address order, waiting for each, so two blocks
    /// taking the same locks can't each hold one the other waits for (D348, D349).
    LockAll {
        addresses: LocalId,
        count: u64,
    },
    /// Gives back the locks a `LockAll` on the same array took (D348).
    UnlockAll {
        addresses: LocalId,
        count: u64,
    },
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
    /// Adds an item at the end of a list (D24). A bounded list checks it has room for one more, and crashes when
    /// it's full unless the failure is ignored, with `None`. A growable list grows with its allocator (D321, D322).
    Push {
        place: Place,
        value: Expression,
        room: Option<(u64, Option<Box<CheckFailure>>)>,
        growth: Option<Growth>,
    },
    /// `std::coll::list.done(Give $xs, alloc: Lend $heap)`: checks the allocator is the list's, and frees it (D322,
    /// D323, D331).
    ListDone {
        place: Place,
        allocator: Expression,
        wrong: Box<CheckFailure>,
    },
    /// `str.done(Give $log, alloc: Lend $heap)`: frees heap text (D326, D331).
    TextDone(Expression),
    /// `std::mem::arena.done(Give $arena, alloc: Lend $heap)`: checks nothing from it is still alive (D330).
    ArenaDone {
        arena: Expression,
        alive: Box<CheckFailure>,
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
    /// An assembly block (D233): values into registers, the raw lines, then registers into locals.
    Assembly {
        lines: Vec<String>,
        reads: Vec<(String, Expression)>,
        writes: Vec<(String, LocalId, MachineType)>,
        clobbers: Vec<String>,
        /// x86's assembly is written in Intel's order, as `mov rcx, rax`.
        intel: bool,
    },
    Check {
        condition: Expression,
        failure: CheckFailure,
        operands: Vec<Expression>,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
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
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
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
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct FloatPrintFailures {
    pub halfway: CheckFailure,
    pub science: CheckFailure,
    pub nan: CheckFailure,
}

/// How a hole prints its value (D194).
#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
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
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Align {
    Left,
    Right,
    Center,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Base {
    #[default]
    Decimal,
    Binary,
    Octal,
    Hex,
    HexUpper,
}

/// What happens when a call fails (standard 8).
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum OnFailure {
    /// Nothing handles it, so the program stops (D96).
    Crash(Vec<Unhandled>),
    /// `?? void` on a call that gives nothing back.
    Ignore,
    /// `?? value`.
    Fallback(Expression),
    /// `?`: the caller's result, with the callee's failure cases moved to the caller's (D47).
    PassUp {
        mapping: Vec<(usize, usize)>,
        #[serde(with = "struct_number")]
        structure: usize,
        #[serde(with = "union_number")]
        union: usize,
        value_type: MachineType,
        /// The deferred statements of every block it leaves, run before it returns (D325).
        cleanup: Block,
    },
    /// `otherwise:` arms, by failure case, giving their value with `instead` into a local.
    Arms { arms: Vec<(usize, Block)>, result: LocalId, union_local: LocalId },
}

/// The crash message for one failure nothing handles.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Unhandled {
    pub case: usize,
    pub failure: CheckFailure,
    /// Whether it carries one value, which its `when:` line prints.
    pub carried: bool,
}

/// What a failed check prints. The message is rendered while building, in the one error layout (standard 11.1),
/// with the values that broke it, if any, printed into its `when:` line while running.
/// How a growable list grows (D321, D322, D370, D385): with which allocator, an item's size and alignment as the
/// standard counts them, how it grows, and what happens when the allocator is the wrong one or out of memory.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Growth {
    pub allocator: Box<Expression>,
    pub standard: u64,
    pub align: u64,
    /// 0 doubles, 1 is exact, 2 steps by `step` items and 3 by `step` bytes.
    pub grow: u8,
    pub step: u64,
    pub wrong: Box<CheckFailure>,
    pub out_of_memory: OutOfMemory,
}

/// What running out of memory does (D321, D324): crash with its message, be ignored with `?? void`, or set a flag
/// for an `otherwise` arm to read.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum OutOfMemory {
    Crash(Box<CheckFailure>),
    Ignore,
    Flag(LocalId),
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(into = "StoredFailure", from = "StoredFailure")]
pub struct CheckFailure {
    /// The text before the `when:` values, or the whole message.
    pub before: String,
    /// `when:` parts: text, then which operand's value, then `after`.
    pub when: Vec<(String, usize)>,
    pub after: String,
    /// The error itself, and its `when:` line's parts, so a check that fails while baking can be refused instead
    /// (D91), and the message can be rendered again where the lines moved (D168).
    pub source: Option<Box<crate::diagnostic::Diagnostic>>,
    pub when_template: Vec<WhenPart>,
    /// `because:` parts added after it, like where an `ensures` was led to by a `return`.
    pub also: Vec<(crate::source::Span, String)>,
}

impl CheckFailure {
    /// Renders a failed check's message from its error, keeping a mark for each value printed while running.
    #[must_use]
    pub fn rendered(
        sources: &crate::source::Sources,
        diagnostic: crate::diagnostic::Diagnostic,
        when_template: Vec<WhenPart>,
        also: Vec<(crate::source::Span, String)>,
    ) -> CheckFailure {
        const MARKS: [char; 8] = ['\u{1}', '\u{2}', '\u{3}', '\u{4}', '\u{5}', '\u{6}', '\u{7}', '\u{8}'];
        let mut template = String::new();
        for part in &when_template {
            match part {
                WhenPart::Text(text) => template.push_str(text),
                WhenPart::Operand(index) => template.push(MARKS[(*index).min(MARKS.len() - 1)]),
            }
        }
        let source = Some(Box::new(diagnostic.clone()));
        let diagnostic = if when_template.is_empty() { diagnostic } else { diagnostic.with_when(template) };
        let rendered = diagnostic.render(sources);
        let mut before = String::new();
        let mut when: Vec<(String, usize)> = Vec::new();
        let mut current = String::new();
        let mut first = true;
        for character in rendered.chars() {
            if let Some(index) = MARKS.iter().position(|mark| *mark == character) {
                if first {
                    before = std::mem::take(&mut current);
                    when.push((String::new(), index));
                    first = false;
                } else {
                    when.push((std::mem::take(&mut current), index));
                }
            } else {
                current.push(character);
            }
        }
        if first {
            before = std::mem::take(&mut current);
        }
        for (span, label) in &also {
            current.push_str(&crate::diagnostic::render_because(sources, *span, label));
        }
        CheckFailure { before, when, after: current, source, when_template, also }
    }
}

/// A failed check as it's stored for later builds.
#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct StoredFailure {
    before: String,
    when: Vec<(String, usize)>,
    after: String,
    source: Option<Box<crate::diagnostic::Diagnostic>>,
    when_template: Vec<WhenPart>,
    also: Vec<(crate::source::Span, String)>,
}

impl From<CheckFailure> for StoredFailure {
    fn from(failure: CheckFailure) -> StoredFailure {
        let CheckFailure { before, when, after, source, when_template, also } = failure;
        StoredFailure { before, when, after, source, when_template, also }
    }
}

impl From<StoredFailure> for CheckFailure {
    fn from(stored: StoredFailure) -> CheckFailure {
        let StoredFailure { before, when, after, source, when_template, also } = stored;
        // Read back with its spans moved to where the function is now, so it's rendered again from there.
        if let Some(diagnostic) = &source
            && let Some(again) = crate::source::relocation::render(|sources| {
                CheckFailure::rendered(sources, (**diagnostic).clone(), when_template.clone(), also.clone())
            })
        {
            return again;
        }
        CheckFailure { before, when, after, source, when_template, also }
    }
}

/// Where a function raises a failure or calls one with `requires`, rendered as the `because:` part of a crash.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(into = "StoredSite", from = "StoredSite")]
pub struct Site {
    pub span: crate::source::Span,
    pub label: String,
    pub text: String,
}

/// A site as it's stored for later builds.
#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct StoredSite {
    span: crate::source::Span,
    label: String,
    text: String,
}

impl From<Site> for StoredSite {
    fn from(site: Site) -> StoredSite {
        StoredSite { span: site.span, label: site.label, text: site.text }
    }
}

impl From<StoredSite> for Site {
    fn from(stored: StoredSite) -> Site {
        let text = crate::source::relocation::render(|sources| {
            crate::diagnostic::render_because(sources, stored.span, &stored.label)
        })
        .unwrap_or(stored.text);
        Site { span: stored.span, label: stored.label, text }
    }
}

/// A part of a `when:` line: text, or an operand's value.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum WhenPart {
    Text(String),
    Operand(usize),
}

/// What an atomic does (D354, D360).
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum AtomicOperation {
    Load,
    /// `%store`: sets the value, giving back nothing (D419).
    Store(Box<Expression>),
    Swap(Box<Expression>),
    /// `%add`, `%sub` and the bit operations, with `.wrap` or `.saturate` where they apply.
    Update {
        operator: Arithmetic,
        value: Box<Expression>,
    },
    /// Sets `value` only if what's there is still `expect`.
    CompareSwap {
        expect: Box<Expression>,
        value: Box<Expression>,
    },
}

/// How an atomic's change is seen by other tasks, strictest last (D354).
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum AtomicOrder {
    Relaxed,
    Acquire,
    Release,
    AcquireRelease,
    SequentiallyConsistent,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
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
    WrappingSubtract,
    WrappingMultiply,
    /// Sticks at the hardware's limits, and a `Clamp` around it sticks at narrower written bounds (D294, D372).
    SaturatingAdd,
    SaturatingSubtract,
    SaturatingMultiply,
    RotateLeft,
    RotateRight,
    BitAnd,
    BitOr,
    BitXor,
    ShiftLeft,
    ShiftRight,
}

/// A `std::math` function CORE-MATH gives, on `f64` (D256).
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum MathsFunction {
    Sin,
    Cos,
    Tan,
    /// One instruction, already correctly rounded, so no CORE-MATH (D405).
    Sqrt,
}

impl MathsFunction {
    pub const ALL: [MathsFunction; 3] = [MathsFunction::Sin, MathsFunction::Cos, MathsFunction::Tan];

    /// CORE-MATH's name for it.
    #[must_use]
    pub fn symbol(self) -> &'static str {
        match self {
            MathsFunction::Sin => "cr_sin",
            MathsFunction::Cos => "cr_cos",
            MathsFunction::Tan => "cr_tan",
            MathsFunction::Sqrt => "llvm.sqrt",
        }
    }

    /// The function `std::math` names this, as in `std::math::sin`.
    #[must_use]
    pub fn from_path(path: &str) -> Option<MathsFunction> {
        match path {
            "std::math::sin" => Some(MathsFunction::Sin),
            "std::math::cos" => Some(MathsFunction::Cos),
            "std::math::tan" => Some(MathsFunction::Tan),
            "std::math::sqrt" => Some(MathsFunction::Sqrt),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum FloatArithmetic {
    Add,
    Subtract,
    Multiply,
    Divide,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Comparison {
    Equal,
    NotEqual,
    Below,
    Above,
    AtLeast,
    AtMost,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Expression {
    pub kind: ExpressionKind,
    pub machine_type: MachineType,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum ExpressionKind {
    /// The address of this function's site, by its position in `sites`.
    Site(usize),
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
        #[serde(with = "function_number")]
        function: FunctionId,
        arguments: Vec<Argument>,
        /// Where it's written, for compile-time work that goes too deep there (D89).
        span: crate::source::Span,
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
        #[serde(with = "union_number")]
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
    /// One of two values, both worked out, picked by a condition: `simd`'s `.%pick` and its lanes' minimum (D302).
    Select {
        condition: Box<Expression>,
        then: Box<Expression>,
        otherwise: Box<Expression>,
    },
    /// A value's bits, widened into the number that holds a C union's bytes (D398).
    UnionBits(Box<Expression>),
    /// A C union's bytes, the first of them read as another type: an integer, a float or a `bool` (D398).
    Bits(Box<Expression>),
    /// A pointer from a number, for hardware registers and memory maps (D315).
    AddressAt(Box<Expression>),
    /// Reading through a pointer, every time and in order when it's volatile (D315, D317).
    PointerRead {
        pointer: Box<Expression>,
        volatile: bool,
    },
    /// Writing through a pointer (D315, D317). Its value is nothing.
    PointerWrite {
        pointer: Box<Expression>,
        value: Box<Expression>,
        volatile: bool,
    },
    /// An atomic's operation on what's at an address, in one indivisible step: its value for `%load`, and what was
    /// there before for the others (D354, D360).
    Atomic {
        address: Box<Expression>,
        operation: AtomicOperation,
        ordering: AtomicOrder,
        /// An overflow check, for a plain `%add` or `%sub` (D354).
        failure: Option<Box<CheckFailure>>,
    },
    /// A pointer moved by a number of bytes, which can be negative (D316).
    PointerMoved {
        pointer: Box<Expression>,
        bytes: Box<Expression>,
    },
    /// Two views of a list split at a place, checked to be inside it or at its end (D347, D382).
    Split {
        list: Place,
        at: Box<Expression>,
        failure: Box<CheckFailure>,
    },
    /// A view of a whole array, as a list record over its items, for a list view parameter (D379).
    ArrayView(Place),
    /// `os::heap()`: the one OS heap (D332, D371).
    Heap,
    /// `os::threads(count: …)`: a pool of the OS's threads, `count` of them, or one per core for 0 (D345, D392).
    Pool {
        count: u64,
    },
    /// Text copied into heap memory of its own, for `str.from` (D326, D369).
    TextCopy(Box<Expression>),
    /// Text checked to fit a bounded string's `len.max` while running, failing otherwise (D326, D328).
    FitText {
        value: Box<Expression>,
        most: u64,
        failure: Box<CheckFailure>,
        /// What `?? value` gives when it doesn't fit, rather than stopping (D132).
        fallback: Option<Box<Expression>>,
    },
    /// `std::mem::arena.new(room: 1 mib, alloc: Lend $heap)` (D330, D333, D386).
    ArenaNew {
        room: u64,
    },
    /// `std::coll::list.from([1, 2, 3], alloc: Lend $heap)`: a growable list filled with items (D369).
    ListFrom {
        items: Vec<Expression>,
        growth: Growth,
    },
    /// One of `std::math`'s correctly rounded functions, from CORE-MATH (D161, D256).
    Maths {
        function: MathsFunction,
        value: Box<Expression>,
    },
    /// A value stuck inside bounds, for saturating maths whose result goes into a bounded type (D294, D372).
    Clamp {
        value: Box<Expression>,
        min: i128,
        max: i128,
    },
    /// A float stuck inside its written bounds, for saturating float maths (D373).
    ClampFloat {
        value: Box<Expression>,
        min: f64,
        max: f64,
    },
    /// Puts a value in a local, then gives `body`, so a value used twice is evaluated once.
    Bind {
        local: LocalId,
        value: Box<Expression>,
        body: Box<Expression>,
    },
}

/// A place in memory: a local, then fields and items inside it (D125, D133).
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Place {
    pub local: LocalId,
    pub steps: Vec<Step>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
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

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
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

/// A function, struct or union a kept body refers to is kept by its name, and gets the number that name has in the
/// build reading it back, since numbers come from the order of declarations (D168).
macro_rules! kept_by_name {
    ($module:ident, $kind:ident) => {
        mod $module {
            use crate::source::relocation::{Named, name_of, number_of};

            #[allow(clippy::trivially_copy_pass_by_ref)]
            pub fn serialize<S: serde::Serializer>(number: &usize, serializer: S) -> Result<S::Ok, S::Error> {
                serializer.serialize_str(&name_of(Named::$kind, *number).unwrap_or_default())
            }

            pub fn deserialize<'de, D: serde::Deserializer<'de>>(deserializer: D) -> Result<usize, D::Error> {
                let name: String = serde::Deserialize::deserialize(deserializer)?;
                number_of(Named::$kind, &name).ok_or_else(|| serde::de::Error::custom("a name that's gone"))
            }
        }
    };
}

kept_by_name!(function_number, Function);
kept_by_name!(struct_number, Struct);
kept_by_name!(union_number, Union);
