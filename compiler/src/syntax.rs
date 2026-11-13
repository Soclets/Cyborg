//! The syntax tree.
//!
//! Every node keeps its span, since every error points at exact source (D16). Names are kept as written, and what
//! they mean is worked out by the checker.

use crate::source::Span;

/// A name as written, with its span.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Name {
    pub text: String,
    pub span: Span,
}

/// What joins a path segment to the one before it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Joint {
    /// The first segment.
    Start,
    /// `::` (D22).
    Path,
    /// `.`, a narrowing or a field (D8).
    Narrowing,
}

/// A method mark on a segment: `%` lends the receiver, `%%` lends it to change (D4, D5), and `*` marks a handler
/// that can jump (D46).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mark {
    None,
    Percent,
    DoublePercent,
    Star,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Segment {
    pub joint: Joint,
    pub mark: Mark,
    /// A limb's `@`, written on its first segment (D36).
    pub limb: bool,
    pub name: Name,
}

/// A path of segments, such as `std::io::fmt_print.stored`, `fn.main`, `time.seconds` or `@csup::union.untagged`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Path {
    pub segments: Vec<Segment>,
    pub span: Span,
}

impl Path {
    /// The path as written, without spaces.
    #[must_use]
    pub fn text(&self) -> String {
        let mut text = String::new();
        for segment in &self.segments {
            match segment.joint {
                Joint::Start => {}
                Joint::Path => text.push_str("::"),
                Joint::Narrowing => text.push('.'),
            }
            if segment.limb {
                text.push('@');
            }
            match segment.mark {
                Mark::None => {}
                Mark::Percent => text.push('%'),
                Mark::DoublePercent => text.push_str("%%"),
                Mark::Star => text.push('*'),
            }
            text.push_str(&segment.name.text);
        }
        text
    }

    #[must_use]
    pub fn first(&self) -> Option<&Segment> {
        self.segments.first()
    }

    #[must_use]
    pub fn last(&self) -> Option<&Segment> {
        self.segments.last()
    }

    #[must_use]
    pub fn is_single(&self, word: &str) -> bool {
        self.segments.len() == 1 && self.segments[0].name.text == word && !self.segments[0].limb
    }
}

/// A whole source file.
#[derive(Clone, Debug, Default)]
pub struct File {
    pub items: Vec<Item>,
}

#[derive(Clone, Debug)]
pub struct Item {
    pub kind: ItemKind,
    pub span: Span,
    /// `pub` in front (D200).
    pub public: Option<Span>,
}

#[derive(Clone, Debug)]
pub enum ItemKind {
    Import(Import),
    Use(Use),
    Function(Function),
    Const(Let),
    Struct(Struct),
    Union(Union),
    /// `fam.unit`, `fam.prefix`, `fam.failure`, `fam.handler`, `fam.type.new` and other families.
    Family(Family),
    /// `type.alias name = T;` or `type.new name = T;` (D134).
    TypeName {
        keyword: Path,
        name: Path,
        target: Type,
    },
    /// `interface ordered<?self>: … end interface ordered` (D235, D238).
    Interface(Requirements),
    /// `type_set whole: i32; i64; end type_set whole` (D235).
    TypeSet(Requirements),
    /// `reading satisfies ordered;`, or the `.|.` easter egg.
    Satisfies {
        subject: Path,
        interface: Path,
    },
    /// A shield at the top of a file, holding declarations that live only in a shield, like C's union (D34, D38).
    Shield {
        name: Name,
        reason: Option<Expression>,
        items: Vec<Item>,
        end: Closer,
    },
    /// A statement where only items go, kept so it can be refused with its span.
    Stray(Statement),
}

#[derive(Clone, Debug)]
pub struct Import {
    /// `pkg` in `import pkg std;` (D188).
    pub whole_package: Option<Span>,
    pub path: Path,
}

#[derive(Clone, Debug)]
pub struct Use {
    pub path: Path,
    /// The short name: a plain name or a quoted one, with any `%` marks written outside the quotes (D93, D151).
    pub short: ShortName,
}

#[derive(Clone, Debug)]
pub struct ShortName {
    pub mark: Mark,
    pub limb: bool,
    /// The name, without quotes.
    pub text: String,
    pub quoted: bool,
    pub span: Span,
    /// The name alone, quotes included, without its `@` or `%` marks.
    pub name_span: Span,
}

/// A block-shaped item kept as raw lines, for things whose insides aren't designed yet.
#[derive(Clone, Debug)]
pub struct Block {
    pub keyword: Path,
    pub name: Path,
    pub span: Span,
}

/// An interface's or type set's lines (D235).
#[derive(Clone, Debug)]
pub struct Requirements {
    pub keyword: Path,
    pub name: Path,
    pub generics: Vec<Generic>,
    pub entries: Vec<RequirementEntry>,
    pub end: Closer,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub enum RequirementEntry {
    /// `includes equatable;`.
    Includes(Path),
    /// `op below [$a: ?self, $b: ?self] -> bool;`, with how many values it takes.
    Op { operator: Name, values: usize, span: Span },
    /// `law …;`, which says what the operations obey.
    Law(Span),
    /// A type on its own line, as a type set lists them.
    Type(Type),
}

#[derive(Clone, Debug)]
pub struct Function {
    /// `fn`, `fn.main`, `fn.pure`, `fn.pure.baked`, or `op` for an operator.
    pub keyword: Path,
    /// The name, empty for `fn.main`. For a method, `reading.%%warm_up`.
    pub name: Option<Path>,
    /// For `op below`, the operator.
    pub operator: Option<Name>,
    pub generics: Vec<Generic>,
    pub parameters: Parameters,
    pub returns: Option<Type>,
    /// `returns` itself, or the `->` written instead.
    pub returns_word: Span,
    pub fails: Option<FailList>,
    pub body: Vec<Statement>,
    /// The closer: `end fn total`.
    pub end: Closer,
    pub header_span: Span,
}

/// The closer of a block, `end if` or `end fn total`, as written.
#[derive(Clone, Debug)]
pub struct Closer {
    pub span: Span,
    pub words: Vec<Name>,
    /// The name after the keyword, as a path.
    pub name: Option<Path>,
    /// The keyword, which can be narrowed, like `fn.main` or `struct.must_Give`.
    pub keyword: Option<Path>,
}

#[derive(Clone, Debug)]
pub struct Generic {
    /// `?item`, a value parameter `$n`, or a borrow source `` `text` ``.
    pub name: Name,
    pub kind: GenericKind,
    pub requirement: Option<Requirement>,
    pub span: Span,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GenericKind {
    Type,
    Source,
}

#[derive(Clone, Debug)]
pub enum Requirement {
    Path(Path),
    And(Box<Requirement>, Box<Requirement>),
    /// `lasts_at_least `text``.
    LastsAtLeast(Name),
}

#[derive(Clone, Debug)]
pub struct Parameters {
    /// `[none]`.
    pub none: Option<Span>,
    pub list: Vec<Parameter>,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct Parameter {
    pub name: Name,
    pub ownership: ParameterOwnership,
    pub ownership_span: Option<Span>,
    pub written_type: Type,
    pub span: Span,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ParameterOwnership {
    /// `Lent`: lent read-only (D393).
    Lent,
    Given,
    LentMut,
    /// No word, for what has no choice of how it's held, like a number or a view (D393).
    Bare,
}

#[derive(Clone, Debug)]
pub struct FailList {
    pub span: Span,
    pub entries: Vec<FailEntry>,
}

#[derive(Clone, Debug)]
pub struct FailEntry {
    pub name: Path,
    pub fields: Vec<Field>,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct Field {
    pub name: Name,
    pub written_type: Type,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct Struct {
    pub keyword: Path,
    pub name: Path,
    pub generics: Vec<Generic>,
    pub fields: Vec<Field>,
    pub end: Closer,
}

#[derive(Clone, Debug)]
pub struct Union {
    pub keyword: Path,
    pub name: Path,
    pub generics: Vec<Generic>,
    pub cases: Vec<Case>,
    pub end: Closer,
}

#[derive(Clone, Debug)]
pub struct Case {
    pub name: Name,
    pub fields: Vec<Field>,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct Family {
    /// `fam.unit`, `fam.failure`, …
    pub keyword: Path,
    pub name: Path,
    pub entries: Vec<FamilyEntry>,
    pub end: Closer,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct FamilyEntry {
    pub name: Path,
    pub kind: FamilyEntryKind,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub enum FamilyEntryKind {
    /// `money.cents: base;`, maybe `with metric`.
    Base { with: Option<Path> },
    /// `money.dollars = 100 money.cents;`, or a prefix's ratio.
    SetTo(Expression),
    /// A failure, maybe with fields: `file_failed.disk_full[free: i32];`.
    Failure { fields: Vec<Field> },
    /// A handler: `word_failed.use_placeholder:` and its arms.
    Handler { arms: Vec<Arm>, end: Closer },
    /// `ids.user = i64;` in `fam.type.new`.
    TypeSetTo(Type),
    /// `money.dollars = 100 money.cents;` or `motion.speed = length.meters per time.seconds;` in `fam.unit`.
    UnitSetTo(UnitDefinition),
    /// `metric.milli = 1 per 1000;` in `fam.prefix`: a prefix's exact scale (D250).
    Prefix { top: Expression, bottom: Option<Expression> },
    /// Nothing after the name.
    Bare,
}

/// What a unit is, after its `=` (D109, D242).
#[derive(Clone, Debug)]
pub enum UnitDefinition {
    /// `100 money.cents`.
    Multiple { amount: Expression, unit: Path },
    /// `length.meters per time.seconds`, `length.meters x length.meters`: each unit with its power.
    Made { factors: Vec<(Path, i32)> },
    /// `temperature.kelvin offset ~-273.15`: the unit it's measured from, and how far its zero is.
    Offset { unit: Path, amount: Expression, span: Span },
    /// `decibels of 1 power.milliwatts`.
    Decibels(Span),
    /// Anything else, such as `1 heat.kelvin + 273`, kept for its error.
    Other(Span),
}

/// A type as written.
#[derive(Clone, Debug)]
pub struct Type {
    pub kind: TypeKind,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub enum TypeKind {
    /// A named type, with what fills it in `< >` and what describes it in `[ ]`: `i32[min: 0, max: 100]`,
    /// `std::coll::list<i32>[len.max: inf items]`, `index<`text`: $article>`.
    Named {
        path: Path,
        fill: Vec<TypeArgument>,
        fill_span: Option<Span>,
        labels: Vec<Label>,
        labels_span: Option<Span>,
    },
    /// `?item`.
    Parameter(Name),
    /// `Lend from $r reading`, a stored borrow (D169).
    Borrow { keyword: Path, sources: Vec<Name>, sources_span: Span, target: Box<Type> },
    /// Something that isn't a type, kept for its error.
    Missing,
}

#[derive(Clone, Debug)]
pub struct TypeArgument {
    pub label: Option<Name>,
    pub value: TypeArgumentValue,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub enum TypeArgumentValue {
    Type(Type),
    /// A borrow source filled in: `` `text`: $article ``.
    Value(Expression),
}

/// A label in brackets: `min: 0`, `len.max: inf items`, `celsius = 21`.
#[derive(Clone, Debug)]
pub struct Label {
    pub name: Path,
    /// `:` for a label, `=` for a field (D57, D283).
    pub sets: bool,
    pub value: Expression,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct Statement {
    pub kind: StatementKind,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub enum StatementKind {
    Let(Let),
    Decl {
        mutable: bool,
        name: Name,
        written_type: Type,
    },
    Assign {
        target: Expression,
        value: Expression,
    },
    Change {
        target: Expression,
        value: Expression,
    },
    Expression(Expression),
    Return(Option<Expression>),
    If(If),
    Match(Match),
    For(For),
    While {
        label: Option<Name>,
        condition: Condition,
        body: Vec<Statement>,
        end: Closer,
    },
    Forever {
        label: Option<Name>,
        body: Vec<Statement>,
        end: Closer,
    },
    Break(Option<Name>),
    Fail(Expression),
    Instead(Expression),
    Finish(Expression),
    /// `defer std::coll::list.done(Give $xs, alloc: Lend $heap);`: runs when its block ends, however it ends (D325).
    Defer(Box<Statement>),
    Labels(Vec<Label>),
    Defaults(Vec<Label>),
    Requires {
        keyword: Path,
        condition: Expression,
    },
    Ensures {
        keyword: Path,
        condition: Expression,
    },
    Shield(Shield),
    /// `on.target aarch64: … else: … end on.target` (D233).
    OnTarget(OnTarget),
    /// `inline.asm: … end inline.asm` (D233).
    Assembly(Assembly),
    /// A statement the parser couldn't read, already reported.
    Broken,
}

#[derive(Clone, Debug)]
pub struct Let {
    /// `let`, `let mut` or `const`.
    pub keyword: Span,
    pub mutable: bool,
    pub constant: bool,
    pub name: Name,
    pub written_type: Option<Type>,
    /// `=`, or `@precog::set_to_best`, or a refused `set_to` or `:=` (D283).
    pub setter: Path,
    pub value: Expression,
}

#[derive(Clone, Debug)]
pub struct If {
    pub branches: Vec<Branch>,
    pub otherwise: Option<(Span, Vec<Statement>)>,
    pub end: Closer,
}

#[derive(Clone, Debug)]
pub struct Branch {
    pub condition: Condition,
    pub body: Vec<Statement>,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub enum Condition {
    Plain(Expression),
    /// `if $s: status.failed:`, the test (D9, D20).
    Test {
        subject: Expression,
        written_type: Type,
        binding: Option<Name>,
    },
}

#[derive(Clone, Debug)]
pub struct Match {
    pub subject: Expression,
    pub arms: Vec<Arm>,
    pub end: Closer,
}

/// A `case` in a `match` or an `otherwise:` block.
#[derive(Clone, Debug)]
pub struct Arm {
    pub pattern: Path,
    pub binding: Option<Name>,
    pub body: Vec<Statement>,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct For {
    pub label: Option<Name>,
    pub name: Name,
    pub ownership: Option<(Ownership, Span)>,
    pub source: Expression,
    pub body: Vec<Statement>,
    /// `leftovers $x:`, the section `simd_chunks.leftovers` hands each leftover item to (D300).
    pub leftovers: Option<(Name, Vec<Statement>)>,
    pub end: Closer,
}

/// `on.target aarch64:`, compiled only when building for that chip (D233).
#[derive(Clone, Debug)]
pub struct OnTarget {
    pub keyword: Path,
    pub chip: Name,
    pub then: Vec<Statement>,
    pub otherwise: Option<Vec<Statement>>,
    pub end: Closer,
}

/// An assembly block: what goes in, what comes out, what else it changes, and its raw lines (D233).
#[derive(Clone, Debug)]
pub struct Assembly {
    /// `inline.asm:`, colon included.
    pub opener: Span,
    pub reads: Vec<(Name, Expression)>,
    pub writes: Vec<(Name, Name)>,
    pub clobbers: Vec<Name>,
    pub lines: Vec<Span>,
}

#[derive(Clone, Debug)]
pub struct Shield {
    pub name: Name,
    pub reason: Option<Expression>,
    pub body: Vec<Statement>,
    pub end: Closer,
}

/// An ownership word on an argument (D2).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ownership {
    Give,
    Copy,
    Lend,
    LendMut,
}

impl Ownership {
    #[must_use]
    pub fn word(self) -> &'static str {
        match self {
            Ownership::Give => "Give",
            Ownership::Copy => "Copy",
            Ownership::Lend => "Lend",
            Ownership::LendMut => "Lend.mut",
        }
    }
}

#[derive(Clone, Debug)]
pub struct Expression {
    pub kind: ExpressionKind,
    pub span: Span,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnaryOperator {
    Negate,
    Not,
    BitNot,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BinaryOperator {
    Add,
    Subtract,
    Multiply,
    /// `div` with its narrowing, if any.
    Divide(Option<String>),
    Remainder(Option<String>),
    Equal,
    NotEqual,
    Below,
    Above,
    AtLeast,
    AtMost,
    And(Option<String>),
    Or(Option<String>),
    BitAnd,
    BitOr,
    BitXor,
    ShiftLeft,
    ShiftRight,
    /// `plus.wrap`, `minus.wrap` and `times.wrap` (D42, D295).
    WrappingAdd,
    WrappingSubtract,
    WrappingMultiply,
    /// `plus.saturate`, `minus.saturate` and `times.saturate`, which stick at the bounds of the type the result
    /// goes into (D294, D372).
    SaturatingAdd,
    SaturatingSubtract,
    SaturatingMultiply,
    /// `rotate.left` and `rotate.right` (D404).
    RotateLeft,
    RotateRight,
    /// `lasts_at_least`, between borrow sources in `requires` (D174).
    LastsAtLeast,
}

impl BinaryOperator {
    #[must_use]
    pub fn is_comparison(&self) -> bool {
        matches!(
            self,
            BinaryOperator::Equal
                | BinaryOperator::NotEqual
                | BinaryOperator::Below
                | BinaryOperator::Above
                | BinaryOperator::AtLeast
                | BinaryOperator::AtMost
        )
    }

    #[must_use]
    pub fn is_bits(&self) -> bool {
        matches!(
            self,
            BinaryOperator::BitAnd
                | BinaryOperator::BitOr
                | BinaryOperator::BitXor
                | BinaryOperator::ShiftLeft
                | BinaryOperator::ShiftRight
                | BinaryOperator::RotateLeft
                | BinaryOperator::RotateRight
        )
    }
}

#[derive(Clone, Debug)]
pub struct Argument {
    pub label: Option<Name>,
    pub ownership: Option<(Ownership, Span)>,
    pub value: Expression,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub enum ExpressionKind {
    Integer(String),
    Float(String),
    /// `~0.1`, an inexact literal (D59), with `exponent` in front if written.
    Inexact {
        literal: Box<Expression>,
        exponent: Option<Span>,
    },
    /// `exponent 6.02e23` with no `~`.
    Exponent(Box<Expression>),
    Text(String),
    Bool(bool),
    Empty,
    Infinity,
    /// `$name`.
    Variable(Name),
    /// A path to something declared: a function, a case, a constant in a kit.
    Path(Path),
    /// A field read: `$r.celsius` (D8).
    Field {
        base: Box<Expression>,
        field: Name,
    },
    /// A call: `total(Lend $xs)`.
    Call {
        callee: Box<Expression>,
        arguments: Vec<Argument>,
        arguments_span: Span,
    },
    /// A call led by a type filled in with `< >`: `ptr.raw.mut<i32>.of(Lend.mut $count)` (D315).
    TypeCall {
        written_type: Type,
        name: Name,
        arguments: Vec<Argument>,
        arguments_span: Span,
    },
    /// A method call: `$xs.std::%%append(val: 4)`.
    MethodCall {
        receiver: Box<Expression>,
        method: Path,
        arguments: Vec<Argument>,
        arguments_span: Span,
    },
    /// A struct, case or failure built with `[ ]`, or an item lookup: `reading[celsius = 21]`,
    /// `$xs[pos: 1]`, `range[from: 1, to: 5]`.
    Bracket {
        base: Box<Expression>,
        labels: Vec<Label>,
        bracket_span: Span,
    },
    /// A list of values: `[1, 2, 3]`.
    List(Vec<Expression>),
    /// `all 0` (D102).
    All(Box<Expression>),
    Unary {
        operator: UnaryOperator,
        operand: Box<Expression>,
        operator_span: Span,
    },
    Binary {
        operator: BinaryOperator,
        operator_span: Span,
        left: Box<Expression>,
        right: Box<Expression>,
    },
    /// A comparison chain, `1 below $x below 5` (D132).
    Chain {
        operands: Vec<Expression>,
        operators: Vec<(BinaryOperator, Span)>,
    },
    /// `( )` grouping (D23).
    Group(Box<Expression>),
    /// A number with its unit: `3600 time.seconds`.
    WithUnit {
        value: Box<Expression>,
        unit: Path,
    },
    /// `$x as T`, maybe narrowed: `as.trunc`.
    As {
        value: Box<Expression>,
        keyword: Path,
        written_type: Type,
    },
    /// `read()?`.
    Pass {
        value: Box<Expression>,
        mark: Span,
    },
    /// `read() ?? 0`.
    Instead {
        value: Box<Expression>,
        fallback: Box<Expression>,
        mark: Span,
    },
    /// `call() otherwise: arms end otherwise`, or `otherwise handler.name`.
    Otherwise {
        value: Box<Expression>,
        keyword: Span,
        handling: Handling,
    },
    /// An ownership word in front of a value, where a value is handed over: `Give $t`, `Lend $r`.
    Owned {
        ownership: Ownership,
        ownership_span: Span,
        value: Box<Expression>,
    },
    /// `bake seconds_in(1)` (D91).
    Bake(Box<Expression>),
    /// A builtin: `@bytes_of<i32>()`.
    Builtin {
        name: Name,
        fill: Vec<TypeArgument>,
        arguments: Vec<Argument>,
    },
    /// `result` in `ensures`.
    Result,
    /// Something the parser couldn't read, already reported.
    Broken,
}

#[derive(Clone, Debug)]
pub enum Handling {
    Arms { arms: Vec<Arm>, end: Closer },
    Handler(Path),
}
