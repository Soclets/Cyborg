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
    pub name: Name,
}

#[derive(Clone, Debug)]
pub struct Use {
    pub path: Path,
    pub short: Name,
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

#[derive(Clone, Debug)]
pub struct FailureEntry {
    pub path: Path,
    pub data: Vec<FieldDeclaration>,
}

#[derive(Clone, Debug)]
pub struct Function {
    pub kind: FunctionKind,
    pub name: Name,
    pub span: Span,
    pub parameters: Vec<Parameter>,
    pub return_type: TypeExpression,
    pub labels: Option<LabelLine>,
    pub defaults: Vec<(Name, Expression)>,
    pub requires: Vec<Contract>,
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
    pub name: Name,
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
    Conversion { unit: Path },
    Unsaid,
}

#[derive(Clone, Debug)]
pub struct UnitMember {
    pub path: Path,
    pub meaning: UnitMeaning,
}

#[derive(Clone, Debug)]
pub struct UnitSet {
    pub family: Name,
    pub members: Vec<UnitMember>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TogetherOperator {
    Multiply,
    Divide,
}

#[derive(Clone, Debug)]
pub struct Together {
    pub left: Path,
    pub operator: TogetherOperator,
    pub right: Path,
    pub result: Path,
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
    Named {
        path: Path,
        fill: Vec<Fill>,
        describe: Vec<Describe>,
    },
    StoredBorrow {
        mutable: bool,
        sources: BorrowSources,
        inner: Box<TypeExpression>,
    },
}

#[derive(Clone, Debug)]
pub enum BorrowSources {
    Program,
    Variables(Vec<Name>),
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

/// One `case` in a `match`, an `otherwise:` or a handler.
#[derive(Clone, Debug)]
pub struct Arm {
    pub pattern: Path,
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BinaryOperator {
    Add,
    Subtract,
    Multiply,
    PlusWrap,
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
    Trunc,
    Floor,
    Exact,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompareOperator {
    Equal,
    NotEqual,
    Less,
    Greater,
    AtLeast,
    AtMost,
}

impl CompareOperator {
    pub fn word(self) -> &'static str {
        match self {
            CompareOperator::Equal => "==",
            CompareOperator::NotEqual => "!=",
            CompareOperator::Less => "<",
            CompareOperator::Greater => ">",
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
        arguments: Vec<Argument>,
    },
    Builtin(Path),
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
