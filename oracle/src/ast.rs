//! What the parser hands to the checker. No spelling lives here: `syntax.rs` is the one place that
//! knows how Cyborg is written.

use crate::report::Span;

/// A dotted or `::` path as written, such as `std::println`, `status.failed` or
/// `i64.std::time.seconds`. It's kept as text so that `use` can rewrite it by prefix.
#[derive(Clone, Debug)]
pub struct Path {
    pub text: String,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct Name {
    pub text: String,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct Program {
    pub imports: Vec<Import>,
    pub uses: Vec<Use>,
    pub items: Vec<Item>,
}

#[derive(Clone, Debug)]
pub struct Import {
    pub limb: bool,
    /// `import pkg std;`, both halves of the package. (D188)
    pub whole: bool,
    pub name: Name,
}

#[derive(Clone, Debug)]
pub struct Use {
    /// The whole line, from `use` to its `;`.
    pub span: Span,
    pub path: Path,
    pub short: Name,
    /// Quoted names are your own choice, and unquoted ones match the path. (D150)
    pub quoted: bool,
    /// `use std::list.%append as %append;`, a kit method. (D178)
    pub method: bool,
}

#[derive(Clone, Debug)]
pub enum Item {
    Function(Function),
    Struct(StructDeclaration),
    Union(UnionDeclaration),
    FailureSet(FailureSet),
    HandlerSet(HandlerSet),
    UnitSet(UnitSet),
    Together(Together),
    Constant(Statement),
    Shield(ShieldItem),
    TypeName(TypeName),
    TypeFamily(Vec<TypeName>),
    Interface(InterfaceDeclaration),
    TypeSet(TypeSetDeclaration),
    Satisfies(Satisfies),
    PrefixSet(PrefixSet),
}

/// `?item: whole and ordered`: a type parameter, and every part of what it needs, which is
/// `any` when it needs nothing. (D234, D235, D239)
#[derive(Clone, Debug)]
pub struct TypeParameter {
    pub name: Name,
    pub requirement: Vec<Path>,
}

/// `interface ordered<?self>:` with its `includes`, `op` and `law` lines. (D235, D238)
#[derive(Clone, Debug)]
pub struct InterfaceDeclaration {
    pub name: Name,
    /// The `?self` its lines are written in terms of.
    pub own: Name,
    pub includes: Vec<Name>,
    pub operators: Vec<InterfaceOperator>,
}

/// `op < [$a: ?self, $b: ?self] -> bool;`, an operation a type must have. (D235, D238)
#[derive(Clone, Debug)]
pub struct InterfaceOperator {
    pub operator: String,
    pub span: Span,
    pub parameters: Vec<Parameter>,
    pub return_type: TypeExpression,
}

/// `type_set whole:` and the types it allows, which nothing joins later. (D235)
#[derive(Clone, Debug)]
pub struct TypeSetDeclaration {
    pub name: Name,
    pub members: Vec<TypeExpression>,
}

/// `reading satisfies ordered;`, or `reading .|. ordered;`, which means the same. (D235)
#[derive(Clone, Debug)]
pub struct Satisfies {
    pub span: Span,
    pub written_type: TypeExpression,
    pub interface: Name,
}

/// `type.alias score = i32;`, or `type.new user_id = i64;` when `new` is true.
#[derive(Clone, Debug)]
pub struct TypeName {
    pub new: bool,
    pub family: Option<String>,
    pub name: Path,
    pub written_type: TypeExpression,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FunctionKind {
    Plain,
    Main,
    Pure,
    PureBaked,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ParameterWord {
    Bare,
    Given,
    LentMut,
}

#[derive(Clone, Debug)]
pub struct Parameter {
    pub name: Name,
    pub word: ParameterWord,
    pub word_span: Span,
    pub written_type: TypeExpression,
}

#[derive(Clone, Debug)]
pub struct LabelLine {
    pub any_order: bool,
    pub entries: Vec<(Name, Name)>,
}

#[derive(Clone, Debug)]
pub struct Contract {
    pub condition: Expression,
}

/// A named source in backticks, as in ``struct index <`text`, `table`: lasts_at_least `text`>``,
/// with the sources it lasts at least as long as. (D171)
#[derive(Clone, Debug)]
pub struct SourceDeclaration {
    pub name: Name,
    pub lasts_at_least: Vec<Name>,
}

/// ``index<`text`: $article>`` fills the source `text` with `$article`. (D171)
#[derive(Clone, Debug)]
pub struct SourceFill {
    pub source: Name,
    pub value: SourceValue,
}

#[derive(Clone, Debug)]
pub enum SourceValue {
    Variable(Name),
    Program,
    /// A function's own source, as in ``view<`text`: `t`>``.
    Source(Name),
}

/// `requires $list lasts_at_least $article;` (D171)
#[derive(Clone, Debug)]
pub struct LastsAtLeast {
    pub longer: Name,
    pub shorter: Name,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct FailureEntry {
    pub path: Path,
    pub data: Vec<FieldDeclaration>,
}

#[derive(Clone, Debug)]
pub struct Function {
    pub kind: FunctionKind,
    /// `op <`'s `<`, for a type's own operator, whose name then says its operator and its
    /// parameters' types, as in `op < [reading, reading]`. (D236)
    pub operator: Option<String>,
    pub name: Name,
    /// `fn largest<?item: ordered>`. (D234)
    pub type_parameters: Vec<TypeParameter>,
    pub span: Span,
    pub parameters: Vec<Parameter>,
    pub return_type: TypeExpression,
    pub labels: Option<LabelLine>,
    pub defaults: Vec<(Name, Expression)>,
    pub requires: Vec<Contract>,
    /// `requires $list lasts_at_least $article;`, which is checked while compiling. (D171)
    pub lasts: Vec<LastsAtLeast>,
    /// ``fn shown <`t`>``, the function's own sources. (D171)
    pub sources: Vec<SourceDeclaration>,
    pub ensures: Vec<Contract>,
    pub fails: Option<Vec<FailureEntry>>,
    pub body: Vec<Statement>,
    pub end_span: Span,
}

#[derive(Clone, Debug)]
pub struct FieldDeclaration {
    pub name: Name,
    pub written_type: TypeExpression,
}

#[derive(Clone, Debug)]
pub struct StructDeclaration {
    pub must_give: bool,
    /// `struct.packed`: the written order with no padding, so a field can't be borrowed. (D319)
    pub packed: bool,
    pub name: Name,
    /// `struct pair<?left: any, ?right: any>:`. (D234)
    pub type_parameters: Vec<TypeParameter>,
    pub sources: Vec<SourceDeclaration>,
    pub fields: Vec<FieldDeclaration>,
}

#[derive(Clone, Debug)]
pub struct UnionCase {
    pub name: Name,
    pub fields: Vec<FieldDeclaration>,
}

#[derive(Clone, Debug)]
pub struct UnionDeclaration {
    pub name: Name,
    pub sources: Vec<SourceDeclaration>,
    pub cases: Vec<UnionCase>,
}

#[derive(Clone, Debug)]
pub struct FailureSet {
    pub name: Name,
    pub members: Vec<FailureEntry>,
}

#[derive(Clone, Debug)]
pub struct Handler {
    pub name: Name,
    pub arms: Vec<Arm>,
}

#[derive(Clone, Debug)]
pub struct HandlerSet {
    pub name: Name,
    pub handlers: Vec<Handler>,
}

#[derive(Clone, Debug)]
pub enum UnitMeaning {
    Base,
    /// `= 100 money.cents`, with its amount.
    Conversion {
        unit: Path,
        amount: Expression,
    },
    /// `length.meters per time.seconds`: what a unit is made of, each part after the first
    /// marked `true` when it divides. (D242)
    Derived {
        parts: Vec<(bool, Path)>,
    },
    /// `temperature.kelvin offset ~-273.15`, a placeholder spelling. (D249)
    Offset {
        unit: Path,
        offset: Expression,
    },
    /// `decibels of 1 power.milliwatts`, a placeholder spelling. (D251)
    Log {
        reference: Path,
        amount: Expression,
    },
    Unsaid,
}

#[derive(Clone, Debug)]
pub struct UnitMember {
    pub path: Path,
    pub meaning: UnitMeaning,
    /// `with metric`: a prefix family whose prefixes make more units from this one. (D250)
    pub prefixes: Option<Name>,
}

/// `fam.prefix metric:` and its prefixes, each a scale, like `metric.kilo = 1000;`. (D250)
#[derive(Clone, Debug)]
pub struct PrefixSet {
    pub family: Name,
    /// Each prefix, with its scale: a number, or one number `per` another.
    pub members: Vec<(Path, Expression, Option<Expression>)>,
}

#[derive(Clone, Debug)]
pub struct UnitSet {
    pub family: Name,
    pub members: Vec<UnitMember>,
}

#[derive(Clone, Debug)]
/// A `together` line, which D242 replaced, so it's only read to be refused.
pub struct Together {
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct ShieldItem {
    pub name: Name,
    pub reason: Option<String>,
    pub items: Vec<ShieldContent>,
}

#[derive(Clone, Debug)]
pub enum ShieldContent {
    UntaggedUnion { path: Path },
    Item(Box<Item>),
}

#[derive(Clone, Debug)]
pub struct TypeExpression {
    pub span: Span,
    pub kind: TypeKind,
}

#[derive(Clone, Debug)]
pub enum TypeKind {
    /// `?item`, a type parameter. (D234)
    Parameter(Name),
    Named {
        path: Path,
        fill: Vec<Fill>,
        describe: Vec<Describe>,
        /// ``index<`text`: $article>``, the struct's sources filled in. (D171)
        sources: Vec<SourceFill>,
    },
    StoredBorrow {
        mutable: bool,
        /// `Lend.held from $r reading` lasts to the end of its block. (D169)
        held: bool,
        sources: BorrowSources,
        inner: Box<TypeExpression>,
    },
}

#[derive(Clone, Debug)]
pub enum BorrowSources {
    Program,
    Variables(Vec<Name>),
    /// ``Lend from `text` reading``, a struct's or function's own source. (D171)
    Source(Name),
}

#[derive(Clone, Debug)]
pub struct Fill {
    pub label: Option<Name>,
    pub written_type: TypeExpression,
}

#[derive(Clone, Debug)]
pub struct Describe {
    pub label: Path,
    pub value: Expression,
}

#[derive(Clone, Debug)]
pub struct Statement {
    pub span: Span,
    pub kind: StatementKind,
}

#[derive(Clone, Debug)]
pub enum StatementKind {
    Let {
        mutable: bool,
        name: Name,
        written_type: TypeExpression,
        value: Expression,
    },
    /// `defer std::list.done(Give $xs, alloc: Lend $heap);`, run when its block ends, however
    /// it ends. (D325)
    Defer(Box<Statement>),
    /// `decl $x: i32;`, a variable with no value yet. (D203)
    Decl {
        mutable: bool,
        name: Name,
        written_type: TypeExpression,
    },
    /// `assign $x = 1;`, its value, once on each path. (D203)
    Assign {
        name: Name,
        value: Expression,
    },
    Constant {
        name: Name,
        written_type: TypeExpression,
        value: Expression,
    },
    Change {
        target: Expression,
        value: Expression,
    },
    /// An `if`, any `else if`s, and maybe an `else:`, closed by one `end if`.
    If {
        links: Vec<IfLink>,
        last: Option<Vec<Statement>>,
    },
    Match {
        subject: Expression,
        arms: Vec<Arm>,
    },
    For {
        loop_name: Option<Name>,
        name: Name,
        source: ForSource,
        body: Vec<Statement>,
        /// `leftovers $x:`, which hands over the items left after a `simd_chunks.leftovers`'s
        /// chunks, one at a time. (D300)
        leftovers: Option<(Name, Vec<Statement>)>,
    },
    While {
        loop_name: Option<Name>,
        condition: Expression,
        body: Vec<Statement>,
    },
    Forever {
        loop_name: Option<Name>,
        body: Vec<Statement>,
    },
    Break {
        name: Option<Path>,
    },
    Finish {
        target: Expression,
    },
    Instead(Expression),
    Return(Expression),
    Fail {
        path: Path,
        data: Vec<FieldValue>,
    },
    Expression(Expression),
    Shield {
        name: Name,
        reason: Option<String>,
        body: Vec<Statement>,
    },
    /// `on.target aarch64:`, compiled only when building for that chip, with `else:` for the
    /// rest. (D233)
    OnTarget {
        /// The oracle builds for no chip, so it never reads which one this is.
        #[allow(dead_code)]
        chip: Name,
        body: Vec<Statement>,
        other: Option<Vec<Statement>>,
    },
    /// `inline.asm:` with what goes in and out, then raw assembly. (D233)
    Assembly(Box<Assembly>),
}

/// An assembly block: registers to fill, registers to read back into new variables, the other
/// registers it changes, and its raw lines. (D233)
#[derive(Clone, Debug)]
pub struct Assembly {
    pub opener: Span,
    pub reads: Vec<(Name, Expression)>,
    pub writes: Vec<(Name, Name)>,
    /// What the assembly changes and says, which LLVM reads and the oracle can't. (D233)
    #[allow(dead_code)]
    pub clobbers: Vec<Name>,
    #[allow(dead_code)]
    pub lines: Vec<(String, Span)>,
}

/// What a `for` goes through.
#[derive(Clone, Debug)]
pub enum ForSource {
    Collection(Expression),
    Range(Box<RangeExpression>),
}

#[derive(Clone, Debug)]
pub struct RangeExpression {
    pub span: Span,
    pub from: Expression,
    pub end: RangeEnd,
    pub step: Option<Expression>,
}

#[derive(Clone, Debug)]
pub struct RangeEnd {
    pub value: Expression,
    /// `to:` includes the end, and `before:` stops before it.
    pub included: bool,
}

#[derive(Clone, Debug)]
pub struct IfLink {
    pub test: IfTest,
    pub body: Vec<Statement>,
}

#[derive(Clone, Debug)]
pub enum IfTest {
    Condition(Expression),
    Type {
        subject: Expression,
        written_type: TypeExpression,
    },
}

/// One `case` in a `match`, an `otherwise:` or a handler. `binding` is the `$f` in `case odd $f:`.
#[derive(Clone, Debug)]
pub struct Arm {
    pub pattern: Path,
    pub binding: Option<Name>,
    pub body: Vec<Statement>,
}

#[derive(Clone, Debug)]
pub struct FieldValue {
    pub name: Name,
    pub value: Expression,
}

#[derive(Clone, Debug)]
pub struct Argument {
    pub label: Option<Name>,
    pub value: Expression,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HandWord {
    Give,
    Copy,
    Lend,
    LendMut,
}

impl HandWord {
    pub fn word(self) -> &'static str {
        match self {
            HandWord::Give => "Give",
            HandWord::Copy => "Copy",
            HandWord::Lend => "Lend",
            HandWord::LendMut => "Lend.mut",
        }
    }
}

/// `plus`, `minus` and `times`: `+`, `-` and `x` written as words, so they can say how they
/// handle overflow. (D295)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WordOperator {
    Plus,
    Minus,
    Times,
}

impl WordOperator {
    pub fn word(self) -> &'static str {
        match self {
            WordOperator::Plus => "plus",
            WordOperator::Minus => "minus",
            WordOperator::Times => "times",
        }
    }

    pub fn symbol(self) -> &'static str {
        match self {
            WordOperator::Plus => "+",
            WordOperator::Minus => "-",
            WordOperator::Times => "x",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BinaryOperator {
    Add,
    Subtract,
    Multiply,
    /// `plus.wrap`, `minus.wrap` and `times.wrap`, which wrap in every mode. (D295)
    Wrap(WordOperator),
    /// `plus.saturate` and the rest, which stick at the type's bounds. (D294, D295)
    Saturate(WordOperator),
    Divide(Narrowing),
    Remainder(Narrowing),
    And,
    Or,
    AndBoth,
    OrBoth,
    BitAnd,
    BitOr,
    BitXor,
    ShiftLeft,
    ShiftRight,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Narrowing {
    /// A bare `div`, which only floats have, and where the word was written. (D164)
    Bare(Span),
    Trunc,
    Floor,
    Exact,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompareOperator {
    Equal,
    NotEqual,
    Below,
    Above,
    AtLeast,
    AtMost,
}

impl CompareOperator {
    pub fn word(self) -> &'static str {
        match self {
            CompareOperator::Equal => "==",
            CompareOperator::NotEqual => "!=",
            CompareOperator::Below => "below",
            CompareOperator::Above => "above",
            CompareOperator::AtLeast => "at_least",
            CompareOperator::AtMost => "at_most",
        }
    }
}

#[derive(Clone, Debug)]
pub enum Handling {
    Block(Vec<Arm>),
    Named {
        family: String,
        name: String,
        jumps: bool,
        span: Span,
    },
}

#[derive(Clone, Debug)]
pub struct Expression {
    pub span: Span,
    pub kind: ExpressionKind,
}

#[derive(Clone, Debug)]
pub enum ExpressionKind {
    Integer {
        digits: String,
        negative: bool,
    },
    Float {
        digits: String,
        negative: bool,
        inexact: bool,
        /// How an exponent literal was written, as in `6.02214076e23`, while `digits` holds its
        /// value in plain digits. (D165)
        exponent: Option<String>,
    },
    Text(String),
    Empty,
    All(Box<Expression>),
    Items(Vec<Expression>),
    Inf,
    Void,
    WithUnit {
        number: Box<Expression>,
        unit: Path,
    },
    Variable(String),
    Field {
        base: Box<Expression>,
        name: Name,
    },
    Path(Path),
    Build {
        path: Path,
        fields: Vec<FieldValue>,
    },
    Call {
        path: Path,
        arguments: Vec<Argument>,
    },
    Method {
        receiver: Box<Expression>,
        kit: Option<String>,
        name: Name,
        percent: bool,
        /// `.%%`, which lends the receiver to change. (D211)
        changes: bool,
        arguments: Vec<Argument>,
    },
    Builtin(Path),
    /// `ptr.raw.mut<u32>.at(address: 0x4004_0000)` or `ptr.raw<i32>.of(Lend $count)`, which
    /// make a pointer, in a shield. (D314, D315, D317)
    Pointer {
        family: Path,
        target: Box<TypeExpression>,
        how: Name,
        arguments: Vec<Argument>,
    },
    Binary {
        operator: BinaryOperator,
        left: Box<Expression>,
        right: Box<Expression>,
    },
    Not(Box<Expression>),
    Negate(Box<Expression>),
    BitNot(Box<Expression>),
    Compare {
        first: Box<Expression>,
        rest: Vec<(CompareOperator, Expression)>,
    },
    As {
        value: Box<Expression>,
        target: TypeExpression,
        /// `as.trunc`, `as.floor` or `as.exact`: how an integer going to a bigger unit
        /// rounds. (D255)
        narrowing: Option<Name>,
    },
    Try(Box<Expression>),
    Fallback {
        value: Box<Expression>,
        fallback: Box<Expression>,
    },
    Otherwise {
        value: Box<Expression>,
        handling: Handling,
    },
    Hand {
        word: HandWord,
        value: Box<Expression>,
    },
    Bake(Box<Expression>),
    Group(Box<Expression>),
    Item {
        base: Box<Expression>,
        position: Box<Expression>,
        counts_from_zero: bool,
    },
}

impl Narrowing {
    /// The narrowing as it's written after `div` or `mod`.
    pub fn word(self) -> &'static str {
        match self {
            Narrowing::Bare(_) => "",
            Narrowing::Trunc => "trunc",
            Narrowing::Floor => "floor",
            Narrowing::Exact => "exact",
        }
    }
}

/// A piece of a format print's text: plain text, or a hole a value goes into. (D194)
#[derive(Clone, Debug)]
pub enum FormatPiece {
    Text(String),
    Hole(Hole),
}

/// `{}`, `{2}` or `{places: 2}`. A numbered hole counts from 1. (D194)
#[derive(Clone, Debug)]
pub struct Hole {
    pub number: Option<usize>,
    /// `{$t}`, a hole that prints a variable by its name. (D210)
    pub name: Option<String>,
    pub shapes: Shapes,
    pub span: Span,
}

/// A hole's shapes, each a word, with where each was written. (D194)
#[derive(Clone, Debug, Default)]
#[allow(clippy::struct_excessive_bools)] // Each bare shape word, like `zeros`, is one flag.
pub struct Shapes {
    pub align: Option<Align>,
    pub width: Option<(usize, WidthUnit)>,
    pub places: Option<usize>,
    pub base: Option<Base>,
    pub science: bool,
    pub zeros: bool,
    pub sign: bool,
    /// `unit`: the value is followed by its unit, the way its file writes it, which the
    /// checker fills in. (D246)
    pub unit: bool,
    pub unit_text: Option<String>,
    pub written: Vec<(&'static str, Span)>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Align {
    Left,
    Right,
    Center,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WidthUnit {
    Cols,
    ColsEastAsian,
    Graphemes,
    Bytes,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Base {
    Bin,
    Oct,
    Hex,
    HexUpper,
}
