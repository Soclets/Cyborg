//! The syntax tree generated programs are built from, and its one printer, written from
//! `standard/`, never from the oracle's or the compiler's source (rule 3, D68, D313).

use std::fmt::Write;

/// The integer types the standard uses: `i32` and `i64` (2.4), and `u8`, `u32` and `u64`,
/// which its rules use too (2.22, 3.8, 2.10's `len`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    I32,
    I64,
    U8,
    U32,
    U64,
}

impl Kind {
    pub const ALL: [Kind; 5] = [Kind::I32, Kind::I64, Kind::U8, Kind::U32, Kind::U64];

    pub fn word(self) -> &'static str {
        match self {
            Kind::I32 => "i32",
            Kind::I64 => "i64",
            Kind::U8 => "u8",
            Kind::U32 => "u32",
            Kind::U64 => "u64",
        }
    }

    /// How many bytes one takes (2.4).
    pub fn bytes(self) -> i128 {
        match self {
            Kind::U8 => 1,
            Kind::I32 | Kind::U32 => 4,
            Kind::I64 | Kind::U64 => 8,
        }
    }

    /// The smallest and largest value the type holds.
    pub fn range(self) -> (i128, i128) {
        match self {
            Kind::I32 => (i128::from(i32::MIN), i128::from(i32::MAX)),
            Kind::I64 => (i128::from(i64::MIN), i128::from(i64::MAX)),
            Kind::U8 => (0, i128::from(u8::MAX)),
            Kind::U32 => (0, i128::from(u32::MAX)),
            Kind::U64 => (0, i128::from(u64::MAX)),
        }
    }

    pub fn bits(self) -> i128 {
        match self {
            Kind::I32 | Kind::U32 => 32,
            Kind::I64 | Kind::U64 => 64,
            Kind::U8 => 8,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Type {
    Integer(Kind),
    Bool,
    /// `f64.stable` when true, `f32.stable` when false: the same bits everywhere (2.6).
    Float(bool),
    /// `i32[min: 0, max: 100]`: still the integer, holding only that range (2.5).
    Bounded(Kind, i128, i128),
    /// `f64.ieee`, allowing NaN and infinity, or `f64.fast`, allowing merged maths, `f64` when
    /// true (2.6).
    OtherFloat(bool, &'static str),
    /// `f64.stable[min: -40.0, max: 125.0]`, a float with whole-number bounds (2.6).
    BoundedFloat(bool, i128, i128),
    /// `str`, a view, or `str[len.max: 8 bytes]`, holding that many bytes inside (2.9).
    Text(Option<i128>),
    /// `simd<i32>[lanes: 4]` (2.21).
    Simd(Kind, i128),
    /// `u32.secret` (2.22).
    Secret(Kind),
    /// `bool.secret`, from comparing secrets, joined only by `and.both`, `or.both` and `not`
    /// (2.22).
    SecretBool,
    /// `f64.stable[unit: fv1.a2]`, or with `.point` or `.span` on a unit with an offset (2.16).
    FloatUnit(&'static str),
    /// `i32[unit: dist.cm]`, or with bounds in units: the bounds as written, and the range in
    /// the type's own unit (2.16).
    Unit(Kind, &'static str, Option<(&'static str, i128, i128)>),
}

impl Type {
    pub fn word(self) -> String {
        match self {
            Type::Integer(kind) => kind.word().to_string(),
            Type::Bool => "bool".to_string(),
            Type::Float(true) => "f64.stable".to_string(),
            Type::Float(false) => "f32.stable".to_string(),
            Type::Bounded(kind, low, high) => format!("{}[min: {low}, max: {high}]", kind.word()),
            Type::OtherFloat(wide, rules) => {
                format!("{}.{rules}", if wide { "f64" } else { "f32" })
            }
            Type::BoundedFloat(wide, low, high) => format!(
                "{}.stable[min: {low}.0, max: {high}.0]",
                if wide { "f64" } else { "f32" }
            ),
            Type::Text(None) => "str".to_string(),
            Type::Text(Some(bytes)) => format!("str[len.max: {bytes} bytes]"),
            Type::Simd(kind, lanes) => format!("simd<{}>[lanes: {lanes}]", kind.word()),
            Type::Secret(kind) => format!("{}.secret", kind.word()),
            Type::SecretBool => "bool.secret".to_string(),
            Type::FloatUnit(unit) => format!("f64.stable[unit: {unit}]"),
            Type::Unit(kind, unit, None) => format!("{}[unit: {unit}]", kind.word()),
            Type::Unit(kind, unit, Some((bounds, ..))) => {
                format!("{}[unit: {unit}, {bounds}]", kind.word())
            }
        }
    }
}

/// Integer maths, each from 3.1's table.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Operator {
    Add,
    Subtract,
    Multiply,
    PlusWrap,
    MinusWrap,
    TimesWrap,
    PlusSaturate,
    MinusSaturate,
    TimesSaturate,
    /// A bare `div`, which is for floats (3.3).
    Divide,
    DivideTrunc,
    DivideFloor,
    RemainderTrunc,
    RemainderFloor,
    AndBit,
    OrBit,
    XorBit,
    ShiftLeft,
    ShiftRight,
}

impl Operator {
    pub const ALL: [Operator; 18] = [
        Operator::Add,
        Operator::Subtract,
        Operator::Multiply,
        Operator::PlusWrap,
        Operator::MinusWrap,
        Operator::TimesWrap,
        Operator::PlusSaturate,
        Operator::MinusSaturate,
        Operator::TimesSaturate,
        Operator::DivideTrunc,
        Operator::DivideFloor,
        Operator::RemainderTrunc,
        Operator::RemainderFloor,
        Operator::AndBit,
        Operator::OrBit,
        Operator::XorBit,
        Operator::ShiftLeft,
        Operator::ShiftRight,
    ];

    pub fn word(self) -> &'static str {
        match self {
            Operator::Add => "+",
            Operator::Subtract => "-",
            Operator::Multiply => "x",
            Operator::PlusWrap => "plus.wrap",
            Operator::MinusWrap => "minus.wrap",
            Operator::TimesWrap => "times.wrap",
            Operator::PlusSaturate => "plus.saturate",
            Operator::MinusSaturate => "minus.saturate",
            Operator::TimesSaturate => "times.saturate",
            Operator::Divide => "div",
            Operator::DivideTrunc => "div.trunc",
            Operator::DivideFloor => "div.floor",
            Operator::RemainderTrunc => "mod.trunc",
            Operator::RemainderFloor => "mod.floor",
            Operator::AndBit => "and.bit",
            Operator::OrBit => "or.bit",
            Operator::XorBit => "xor.bit",
            Operator::ShiftLeft => "shl",
            Operator::ShiftRight => "shr",
        }
    }

    pub fn divides(self) -> bool {
        matches!(
            self,
            Operator::Divide
                | Operator::DivideTrunc
                | Operator::DivideFloor
                | Operator::RemainderTrunc
                | Operator::RemainderFloor
        )
    }

    pub fn is_shift(self) -> bool {
        matches!(self, Operator::ShiftLeft | Operator::ShiftRight)
    }

    /// Its level in 3.9's table, tightest first, for the operators the table lists. Bit
    /// operators, shifts and the `.wrap` and `.saturate` words aren't in it, so they're
    /// always grouped.
    pub fn level(self) -> Option<u8> {
        match self {
            Operator::Multiply
            | Operator::Divide
            | Operator::DivideTrunc
            | Operator::DivideFloor
            | Operator::RemainderTrunc
            | Operator::RemainderFloor => Some(3),
            Operator::Add | Operator::Subtract => Some(4),
            _ => None,
        }
    }
}

/// The comparisons, from 3.5.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Comparison {
    Equal,
    NotEqual,
    Below,
    Above,
    AtLeast,
    AtMost,
}

impl Comparison {
    pub const ALL: [Comparison; 6] = [
        Comparison::Equal,
        Comparison::NotEqual,
        Comparison::Below,
        Comparison::Above,
        Comparison::AtLeast,
        Comparison::AtMost,
    ];

    pub fn word(self) -> &'static str {
        match self {
            Comparison::Equal => "==",
            Comparison::NotEqual => "!=",
            Comparison::Below => "below",
            Comparison::Above => "above",
            Comparison::AtLeast => "at_least",
            Comparison::AtMost => "at_most",
        }
    }
}

#[derive(Clone, Debug)]
pub enum Expression {
    Integer(i128),
    /// A float literal as written, `1.25` or `~0.1`, which is negative when it starts with `-`
    /// or `~-` (2.7).
    Float(String),
    /// A string literal's text, `empty` when it has none (2.9).
    Text(String),
    /// A `simd`'s lanes, `[1, 2, 3, 4]`, or `all 0` (2.21).
    Lanes(Vec<Expression>),
    AllLanes(Box<Expression>),
    /// A `.%` method with no arguments, like `$v.%sum()` or `($a below $b).%any_true()` (2.21).
    Method {
        receiver: Box<Expression>,
        method: &'static str,
    },
    /// `($a == $b).%pick(if_true: …, if_false: …)`, choosing with no branch (2.22).
    Pick {
        test: Box<Expression>,
        if_true: Box<Expression>,
        if_false: Box<Expression>,
    },
    /// `not.bit $x`, every bit flipped (3.7).
    NotBit(Box<Expression>),
    /// A named shuffle or a lookup on a `simd`: `$v.%reverse()`, `$v.%rotate.left(by: 1)`,
    /// `$a.%join(then: $b)`, `$a.%interleave(with: $b)` or `$t.%lookup(pos: $which)` (2.21).
    SimdMethod {
        receiver: String,
        method: &'static str,
        argument: Option<(&'static str, Box<Expression>)>,
    },
    /// `$v.%shuffle(pos: [4, 3, 2, 1])`, or with `offset.lanes:`, a new vector from the lanes
    /// listed (2.21).
    Shuffle {
        receiver: String,
        lanes: Vec<i128>,
        from_zero: bool,
    },
    /// A number with its unit behind it, `250 dist.cm` (2.16).
    WithUnit {
        number: i128,
        unit: &'static str,
    },
    /// A float with its unit behind it, `2.5 fv1.a2` (2.16).
    FloatWithUnit {
        number: String,
        unit: &'static str,
    },
    /// `$v as i32[unit: dist.mm]`, `$v as.trunc …` or `$v as i32`, changing or taking off a
    /// unit (2.16).
    Convert {
        variable: String,
        narrowing: &'static str,
        target: Type,
    },
    /// `$v as user_id`, crossing into or out of a named type (2.18).
    AsNamed {
        variable: String,
        written: String,
    },
    /// `$v as str[len.max: 11 bytes]`, on a single value (2.14, 3.9).
    As {
        variable: String,
        target: Type,
    },
    /// `($n as str[len.max: 2 bytes]) ?? "no"`: a failed `as` handled with a value instead, the
    /// `as` grouped, since it goes on a single value (2.14, 8.5).
    AsOrElse {
        value: Box<Expression>,
        instead: Box<Expression>,
    },
    Bool(bool),
    Variable(String),
    /// A struct built in place, `rec3[fd4 = 1, fd5 = 2]` (7.2).
    Build {
        struct_name: String,
        fields: Vec<(String, Expression)>,
    },
    /// `bake …`, a call to a `fn.pure.baked` function worked out while compiling (5.11).
    Bake(Box<Expression>),
    /// A variable lent to a call, `Lend $r` (6.4).
    Lend(String),
    /// A variable lent to a call to change, `Lend.mut $l` (6.5).
    LendMut(String),
    Binary {
        operator: Operator,
        left: Box<Expression>,
        right: Box<Expression>,
    },
    Compare {
        comparison: Comparison,
        left: Box<Expression>,
        right: Box<Expression>,
    },
    /// `and` when true, `or` when false.
    Logic {
        and: bool,
        /// `and.both` or `or.both`, which run both sides (3.6).
        both: bool,
        left: Box<Expression>,
        right: Box<Expression>,
    },
    Not(Box<Expression>),
    Negate(Box<Expression>),
    /// A comparison chain, `1 below $x below 5`, which means every comparison holds and reads
    /// each value once (3.9).
    Chain {
        values: Vec<Expression>,
        comparisons: Vec<Comparison>,
    },
    Call {
        function: String,
        /// Each argument with its label, if its parameter has one (5.2).
        arguments: Arguments,
        handling: Handling,
    },
    /// An array's item, `$a[pos: 2]` or `$a[offset.items: 1]` (2.17).
    Item {
        array: String,
        position: Position,
    },
    /// A reading method called on a struct variable, `$r.%name()` (5.6).
    CallMethod {
        receiver: String,
        name: String,
    },
    /// A struct's field, `$r.f3` (7.3).
    Field {
        variable: String,
        field: String,
    },
}

/// A call's arguments, each with its label if its parameter has one (5.2).
pub type Arguments = Vec<(Option<String>, Expression)>;

/// What a call does if it fails: nothing, so the program stops (8.9), `?` to pass it up (8.4),
/// or `??` and a value to use instead (8.5).
#[derive(Clone, Debug)]
pub enum Handling {
    Crash,
    Pass,
    Fallback(Box<Expression>),
}

/// How a struct lays out its fields, each a narrowing but the bare one (7.1).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Layout {
    Written,
    Compact,
    AnyOrder,
    Packed,
}

impl Layout {
    pub const ALL: [Layout; 4] = [
        Layout::Written,
        Layout::Compact,
        Layout::AnyOrder,
        Layout::Packed,
    ];

    pub fn word(self) -> &'static str {
        match self {
            Layout::Written => "struct",
            Layout::Compact => "struct.compact",
            Layout::AnyOrder => "struct.any_order",
            Layout::Packed => "struct.packed",
        }
    }
}

/// A family of units: each a base, a whole number of another, made of others with `per` or
/// `x`, or a base with an offset (2.16).
#[derive(Clone, Debug)]
pub struct UnitFamily {
    /// `fam.unit`, or `fam.prefix` for a family of prefixes.
    pub keyword: &'static str,
    pub name: String,
    /// Each unit's full name, and what follows it on its line, as `: base` or `= 12 fu1.b2`.
    pub units: Vec<(String, String)>,
}

/// A named integer type: `type.alias`, fully interchangeable, or `type.new`, which doesn't mix,
/// alone or in a `fam.type.new` family, where it's used as `family.member` (2.18).
#[derive(Clone, Debug)]
pub struct NamedType {
    pub name: String,
    pub alias: bool,
    pub family: Option<String>,
    pub kind: Kind,
}

impl NamedType {
    /// How it's written where it's used.
    pub fn used(&self) -> String {
        match &self.family {
            Some(family) => format!("{family}.{}", self.name),
            None => self.name.clone(),
        }
    }
}

/// A parameter of a void function: an integer lent to change (`Lent.mut`), a plain integer, or
/// a struct, `Lent` or `Given` (6.1, 6.5, D393).
#[derive(Clone, Debug)]
pub enum ParamKind {
    Changed(Kind),
    Plain(Kind),
    Record { struct_name: String, given: bool },
}

/// A void function, called as a statement (5.3, 6.1).
#[derive(Clone, Debug)]
pub struct Procedure {
    pub name: String,
    pub parameters: Vec<(String, ParamKind)>,
    pub body: Vec<Statement>,
}

/// A method on a struct: reading, `fn rec.%name [$r: rec] -> i32`, giving back a value from its
/// fields, or changing, `fn rec.%%name [$r: Lent.mut rec, $by: i32] -> void`, changing a field
/// (5.6).
#[derive(Clone, Debug)]
pub struct MethodDecl {
    pub struct_name: String,
    pub name: String,
    /// The changed field, for a `%%` method.
    pub changes: Option<String>,
    /// Whether its `$by` has the label `by:`, which calls write (5.2, 5.6).
    pub labelled: bool,
    pub kind: Kind,
    pub value: Expression,
}

/// A tagged union, each case with integer data or none (7.5).
#[derive(Clone, Debug)]
pub struct UnionDecl {
    pub name: String,
    pub cases: Vec<(String, Vec<(String, Kind)>)>,
    /// Whether it has `.%size`, a method that matches it and gives an `i32` from each case
    /// (5.6, 4.3).
    pub sized: bool,
}

/// A struct with integer fields (7.1).
#[derive(Clone, Debug)]
pub struct StructDecl {
    pub name: String,
    pub layout: Layout,
    pub fields: Vec<(String, Kind)>,
    /// The fields that are bounded, with their least and greatest values (2.5).
    pub bounds: Vec<(String, i128, i128)>,
}

impl StructDecl {
    /// A field's type: its kind, bounded if it's one of the bounded fields.
    pub fn field_type(&self, field: &str, kind: Kind) -> Type {
        self.bounds
            .iter()
            .find(|(found, ..)| found == field)
            .map_or(Type::Integer(kind), |(_, low, high)| {
                Type::Bounded(kind, *low, *high)
            })
    }
}

/// A `must_Give` struct, `tok…`, with integer fields, ended by `tend`, which `finish`es it, made
/// by `tmake` and handed through by `tpass`, and a `must_Give` holder of one, closed by `tclose`,
/// which moves the field out and then `finish`es the holder (6.9).
#[derive(Clone, Debug)]
pub struct Tokens {
    pub name: String,
    pub fields: Vec<(String, Kind)>,
    pub holder: String,
}

/// How a `must_Give` value is handed off on every path (6.9, 6.12).
#[derive(Clone, Debug)]
pub enum Ending {
    /// `tend(Give $t);` straight after.
    Direct,
    /// `defer tend(Give $t);` right after it's made.
    Deferred,
    /// `tend(Give $t);` in both arms of an `if`.
    Branches(Expression),
    /// Made by `tmake`, handed through `tpass` into a new name, then ended.
    Passed {
        made_from: Expression,
        passed: String,
    },
    /// Inside a holder, closed by `tclose`.
    Held,
    /// Two to four of them in an array, each taken by `for … in Give` and ended (4.4).
    Looped { count: i128, item: String },
    /// Ended in every arm of a `match` on a union (4.3, 6.9).
    Matched {
        variable: String,
        union: String,
        cases: Vec<String>,
    },
}

/// What a program declares for generics: the interfaces `equatable`, `ordered` and `addable`, the
/// `type_set` `whole` of `i32` and `i64`, which types say they satisfy which, a struct with its
/// own `below`, `==` and `+`, a generic struct `gbox`, and generic functions (3.10, 5.8).
#[derive(Clone, Debug)]
pub struct Generics {
    /// Integer types that say they satisfy `ordered`.
    pub ordered: Vec<Kind>,
    /// A struct with its own operators, which satisfies `ordered` and `addable`.
    pub structure: Option<StructDecl>,
    /// Which `satisfies` line, counting from 0, is written as `.|.` instead.
    pub easter: usize,
    /// Whether `glarger`, with a unit parameter, is declared.
    pub unit_parameter: bool,
}

/// A failure a function lists, `bad3` or `bad4[got: i32]` (8.1).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FailureDecl {
    /// Its name, or its full name, `fs1.a2`, in a named set.
    pub name: String,
    pub data: Option<Kind>,
    /// The named set it's in, which a function lists whole (8.3).
    pub family: Option<String>,
}

/// What an `otherwise` arm does (8.6).
#[derive(Clone, Debug)]
pub enum Arm {
    /// `instead …;`, the value used in place of the failed call.
    Instead(Expression),
    /// `return …;` from the function, `return void;` in a `void` one.
    Return(Option<Expression>),
    /// `break;` out of the loop around.
    Break,
    /// `fail …;`, one of the function's own failures, with data if it carries some.
    Fail(String, Option<Expression>),
}

/// A named set of failures, and handler narrowings for it: `fallback`, whose arms give a value
/// instead, and `bail`, whose arms `return void;`, used with `*` (8.3, 8.7, 8.8).
#[derive(Clone, Debug)]
pub struct FailureFamily {
    pub name: String,
    pub members: Vec<FailureDecl>,
    pub handler: String,
}

/// Where an item is: `pos:` counts from 1, and `offset.items:` from 0 (2.17). A variable's
/// value may be outside the array, which is a check while running.
#[derive(Clone, Debug)]
pub enum Position {
    Pos(i128),
    Offset(i128),
    PosOf(String),
}

/// How an array gets its items: one each from `[ ]`, or `all` of one value (2.11).
#[derive(Clone, Debug)]
pub enum Filling {
    Items(Vec<Expression>),
    All(Expression),
}

#[derive(Clone, Debug)]
pub enum Statement {
    Let {
        name: String,
        value_type: Type,
        value: Expression,
    },
    Change {
        name: String,
        value: Expression,
    },
    /// `let mut $a: std::coll::array<i32>[len: 3 items] = [1, 2, 3];` (2.11)
    LetArray {
        name: String,
        kind: Kind,
        len: i128,
        filling: Filling,
    },
    /// `let mut $r: rec1 = rec1[f2 = …, f3 = …];` (7.2)
    LetStruct {
        name: String,
        struct_name: String,
        fields: Vec<(String, Expression)>,
    },
    /// `change $r.f2 = …;` (7.3)
    ChangeField {
        variable: String,
        field: String,
        value: Expression,
    },
    /// `let mut $l: std::coll::list<i32>[len.max: 4 items] = [1, 2];`, or `= empty` (2.10)
    LetList {
        name: String,
        kind: Kind,
        max: i128,
        items: Vec<Expression>,
        /// For a list that grows: the allocator it takes memory from, `heap` or `arena`, with
        /// `len.max: inf items`, `list.from`, and a `defer` that ends it (D321, D323, D369).
        allocator: Option<String>,
        /// How a growable list grows, as `exact` or `by 4 items`, when it says (2.10).
        grow: Option<String>,
    },
    /// A string on the heap, `str.from(…, alloc: Lend $heap)`, from a literal or a number, ended
    /// by a deferred `str.done` and printed through a named hole (2.9, 2.14).
    HeapText {
        name: String,
        from: Expression,
    },
    /// `let $heap: os::heap = os::heap();`, so a function's lists can grow (2.10).
    Heap,
    /// A call made for what it does, `vbump_i32(Lend.mut $l);` (2.10).
    Call(Expression),
    /// `let $h: std::coll::halves<i32> = $l.std::%split(at.items: …);`, at its start or its end
    /// (2.10, D382).
    Split {
        name: String,
        kind: Kind,
        list: String,
        at_end: bool,
    },
    /// `$l.std::%%append(val: …);`, which stops the program when the list is full, or for a
    /// list that grows, names its allocator and can run out of memory (2.10, 8.9, D321).
    Append {
        /// `?? void`, so a full list or a lack of memory just skips the append (8.5).
        handled: bool,
        list: String,
        value: Expression,
        allocator: Option<String>,
    },
    /// `change $a[pos: 2] = …;` (2.17)
    ChangeItem {
        array: String,
        position: Position,
        value: Expression,
    },
    Print(Expression),
    /// A print with its own format text and values (1.13).
    Format {
        /// Where it goes: `out`, or `main`'s `err` and `buf` when it has them (1.13).
        writer: &'static str,
        /// `fmt_print.stored`, which prints every float exactly as it's stored (1.13).
        stored: bool,
        text: String,
        values: Vec<Expression>,
    },
    /// `$buf.%%flush();`, which pushes a buffered writer's output now (1.13).
    Flush,
    If {
        test: Expression,
        then: Vec<Statement>,
        otherwise: Vec<Statement>,
    },
    /// A loop that counts `$counter` from 0 to `rounds`, so it always ends.
    While {
        counter: String,
        rounds: i128,
        body: Vec<Statement>,
    },
    /// `for $counter in range[from: …, to: …, step_by: …]:` (4.4, 4.5), its counter unused,
    /// since what type a literal range's counter is isn't stated.
    For {
        /// The loop's name, which a `break` can name to stop it and every loop inside (4.8).
        name: Option<String>,
        counter: String,
        from: i128,
        /// A literal, or a value only known while running (4.5).
        end: Expression,
        /// `to:` when true, `before:` when false.
        inclusive: bool,
        step: Option<i128>,
        body: Vec<Statement>,
    },
    /// `break;`, which stops the innermost loop (4.8).
    Break,
    /// `break name;`, which stops the named loop and every loop inside it (4.8).
    BreakNamed(String),
    /// `for $x in $a:`, reading each item, or `for $x in Lend.mut $a:`, changing each (4.4).
    ForEach {
        name: Option<String>,
        item: String,
        array: String,
        changes: bool,
        body: Vec<Statement>,
    },
    /// `decl mut $d: i32;` then an `if` that gives it a value on each path with `assign`
    /// (2.19).
    DeclAssign {
        name: String,
        /// `decl mut`, which can change after, or `decl`, which can't.
        mutable: bool,
        kind: Kind,
        test: Expression,
        then_value: Expression,
        else_value: Expression,
    },
    /// `decl $d: i32;` or `decl mut`, with no value yet (2.19).
    Decl {
        name: String,
        mutable: bool,
        kind: Kind,
    },
    /// `assign $d = …;`, a `decl`'s value, once on each path (2.19).
    Assign {
        name: String,
        value: Expression,
    },
    /// Statements that belong together, printed one after another.
    Sequence(Vec<Statement>),
    /// `const $k: i32 = 60 times.wrap 60;`, known when compiling (2.3).
    Const {
        name: String,
        /// An integer, or a `.stable` float (2.3).
        value_type: Type,
        value: Expression,
    },
    /// `let mut $u: u1 = u1.c3[g4 = …];` (7.5)
    LetCase {
        name: String,
        union_name: String,
        case: String,
        fields: Vec<(String, Expression)>,
    },
    /// `match $u:` with an arm for every case, naming its data when it has some (4.3).
    Match {
        variable: String,
        union_name: String,
        arms: Vec<(String, Option<String>, Vec<Statement>)>,
    },
    /// A call started as a task on `main`'s one-thread runner, then ended, its result going
    /// into a new variable (D356, D357, D376).
    Task {
        task: String,
        name: String,
        kind: Kind,
        call: Expression,
        /// `runner`, `main`'s one-thread runner, or `pool` or `events` when it has them.
        runner: &'static str,
    },
    /// `let mut $w: wrap3 = Copy $r as wrap3;`, a struct crossing into a `type.new` over it,
    /// then out again with `Give … as`, into a struct variable (2.18).
    WrapStruct {
        wrapped: String,
        new_type: String,
        word: &'static str,
        source: String,
        back: String,
        struct_name: String,
    },
    /// `let mut $v: al3 = …;`, or for a `type.new`, a value crossing in with a literal or `as`,
    /// then back out into a plain variable with `as` (2.18).
    LetNamed {
        name: String,
        written: String,
        value: Expression,
        /// For a `type.new`: the plain variable it crosses back out into, and its kind.
        back: Option<(String, Kind)>,
    },
    /// `shield …: reason "…" change $v = $s.%reveal(); end shield …`: a secret made public,
    /// which only a shield may do (2.22, 9.7).
    Reveal {
        shield: String,
        target: String,
        secret: String,
    },
    /// A C union built in a shield by naming one field and `rest = all 0`, and any field read
    /// back into a variable that can change, from the same bytes (9.6, D398, D402).
    Untagged {
        shield: String,
        name: String,
        union: String,
        written: (String, Expression),
        read: String,
        read_into: String,
    },
    /// A `must_Give` value made and handed off on every path, one field read into a variable
    /// that can change on the way (6.9).
    Token {
        name: String,
        struct_name: String,
        holder: String,
        fields: Vec<(String, Expression)>,
        ending: Ending,
        read_into: Option<(String, String)>,
    },
    /// A list split into halves, each summed by its own task reading it, the two results joined
    /// and added into a variable (D343, D382).
    SplitTasks {
        halves: String,
        list: String,
        kind: Kind,
        at_end: bool,
        tasks: (String, String),
        runner: &'static str,
        target: String,
    },
    /// An atomic: made, added to with wrapping, swapped, compared and swapped with the lost race
    /// handled, and read into a variable (D361, D378).
    Atomic {
        name: String,
        kind: Kind,
        numbers: (i128, i128, i128),
        old: String,
        now: String,
        /// Whether the read says the weaker `.acquire` ordering (D361).
        acquire: bool,
    },
    /// One or two locks, taken together in one block, which changes each and copies the first
    /// into a variable (D358, D359).
    Locks {
        locks: Vec<(String, i128)>,
        bound: Vec<String>,
        kind: Kind,
        target: String,
    },
    /// `for $x in trio_i32(…):`, a loop over a fresh array a call returns, which takes no word,
    /// adding each item to a variable (4.4, D396).
    FreshLoop {
        item: String,
        call: Expression,
        target: String,
    },
    /// A stored borrow of a struct: `Lend from $r rec`, read; `Lend.held from`, read, in a
    /// block of its own, since nothing changes the struct until that block ends; `Lend.mut
    /// from`, changing a field; or one `rwarm` returns from two structs (6.10).
    BorrowStruct {
        how: &'static str,
        view: String,
        records: (String, String),
        struct_name: String,
        field: String,
        target: String,
        /// The kind of a new variable the field is read into, or none to change `target`.
        declare: Option<Kind>,
        value: Expression,
    },
    /// `defer` on a print or a `change`, which runs when the block ends (6.12).
    Defer(Box<Statement>),
    /// A loop over an array or list in `simd`-sized chunks, saying how the leftovers go:
    /// padded, exact, or handed over one at a time after the chunks; each chunk's greatest lane
    /// is added to a variable, and each leftover too (2.21).
    Chunks {
        array: String,
        lanes: i128,
        /// `pad`, `exact` or `leftovers`.
        mode: &'static str,
        chunk: String,
        leftover: String,
        target: String,
    },
    /// A map, `empty`, ended with `std::coll::map.done` by a `defer` (2.12).
    Map {
        name: String,
        /// `hash` or `ordered`.
        kind: &'static str,
        /// The key's type, as written.
        key: String,
        value: Kind,
    },
    /// `let $c: Lend from <$a, $b> str = tpick($a, $b, …);`, a stored borrow of one of two views,
    /// printed (2.9, 6.10).
    PickText {
        name: String,
        left: String,
        right: String,
        test: Expression,
    },
    /// A struct made by a value: a generic call or the struct's own operator (3.10, 5.8).
    LetStructValue {
        name: String,
        struct_name: String,
        value: Expression,
    },
    /// A generic struct with two type parameters, `gpair<i32, rec3>`, holding an integer and an
    /// integer or a struct, given to `gfirst`, which has two type parameters too, for the first
    /// (5.8).
    GenericPair {
        name: String,
        left: Kind,
        /// The second's type, as written.
        right: String,
        first: Expression,
        second: Expression,
        read_into: String,
    },
    /// A generic struct holding an integer, `gbox<i32>[held = …]`, read back into a variable
    /// (5.8).
    GenericBox {
        name: String,
        kind: Kind,
        value: Expression,
        read_into: String,
    },
    /// One aarch64 instruction in a shield, in an `on.target aarch64:` branch, with `else:`
    /// working out the same value for every other chip, so the two have to agree (9.4).
    Assembly {
        shield: String,
        result: String,
        /// The instruction, as `madd x0, x0, x1, x2`.
        instruction: &'static str,
        /// The `i64` variables put in `x0`, `x1` and on.
        reads: Vec<String>,
        otherwise: Expression,
    },
    /// A shield going through a pointer to an integer variable: made with `.of`, written
    /// through when it can write, moved some items on and the same bytes back, and read into
    /// another variable or printed (9.7, 9.8).
    Pointer {
        shield: String,
        /// `raw` or `volatile`.
        family: &'static str,
        /// Whether it's `.mut`, made with `Lend.mut` and able to write.
        writes: Option<i128>,
        kind: Kind,
        pointer: String,
        target: String,
        /// How many items it moves on and back, and the names of the two moved pointers.
        moves: Option<(i128, String, String)>,
        /// Where the read goes: a variable, or a print when `None`.
        read_into: Option<String>,
        /// Whether it's `.mut` even without a write.
        mutable: bool,
    },
    /// The test, `if $w: un1.cs2:` or `if $v: i32[min: 0, max: 9]:`, inside which the value is
    /// taken as that case or bound (4.2).
    Test {
        variable: String,
        written: String,
        body: Vec<Statement>,
    },
    /// A changing method called on a struct variable, `$r.%%name(…);` (5.6).
    CallChanging {
        receiver: String,
        name: String,
        value: Expression,
        /// Whether the method's `$by` has the label `by:` (5.6).
        labelled: bool,
    },
    /// A call to a void function, its arguments with their ownership words (6.1).
    CallProcedure {
        name: String,
        arguments: Vec<String>,
    },
    /// `forever:`, which a counter always stops with `break` (4.7, 4.8).
    Forever {
        counter: String,
        rounds: i128,
        body: Vec<Statement>,
    },
    /// `fail bad3;` or `fail bad4[got = $p2];` (8.2).
    Fail {
        name: String,
        data: Option<Expression>,
    },
    /// A `let` whose call's failures are each handled with `instead` (8.6).
    LetOtherwise {
        name: String,
        value_type: Type,
        call: Expression,
        /// Each failure, the name an arm reads its data through, and what the arm does.
        arms: Vec<(String, Option<String>, Arm)>,
        /// A handler narrowing used instead of arms, as `hs1.fallback` or `hs1.*bail` (8.7, 8.8).
        handler: Option<String>,
    },
}

#[derive(Clone, Debug)]
pub struct Function {
    pub name: String,
    pub parameters: Vec<(String, Type)>,
    /// Labels on the last parameters, which every call writes (5.2, 5.4).
    pub labels: Vec<(String, String)>,
    /// `labels.any_order`, so calls may write labelled arguments in any order (5.2).
    pub any_order: bool,
    /// Defaults for labelled parameters, which calls may leave off (5.4).
    pub defaults: Vec<(String, Expression)>,
    /// `fn.pure`: the same output for the same input, changing nothing else (5.9).
    pub pure: bool,
    /// `requires …;` on the parameters, checked at runtime (5.5).
    pub requires: Option<Expression>,
    /// `ensures result … ;` (5.4, 5.5).
    pub ensures: Option<(Comparison, Expression)>,
    pub result: Type,
    pub failures: Vec<FailureDecl>,
    pub body: Vec<Statement>,
    pub returned: Expression,
}

#[derive(Clone, Debug)]
#[allow(clippy::struct_excessive_bools)] // Independent switches for what a program declares.
pub struct Program {
    pub unit_families: Vec<UnitFamily>,
    /// `nest`, holding the first struct, an `i32` and a bounded list of this kind, and `view`,
    /// holding a borrow of the first struct, when they're declared (7.1, 6.10).
    pub nest: Option<(String, Kind)>,
    /// The struct `rwarm` takes two of, returning a borrow of one, when it's declared (6.10).
    pub warm: Option<(String, String)>,
    /// The kinds that have `trio_…`, which returns a fresh array of three (4.4, D396).
    pub trios: Vec<Kind>,
    /// Whether `main` also has `$pool`, an `os::threads`, and `$events`, an `os::events`, each
    /// ended by a deferred `os::done` (D346, D392).
    pub runners: bool,
    /// Names for things other than integers: each name, whether it's an alias, and what it
    /// names, as written (2.18).
    pub named_others: Vec<(String, bool, String)>,
    /// Whether `tag`, a struct with a `str` field, `tpick`, which returns a borrow of one of
    /// two views, and `tcount`, which takes one, are declared (2.9, 6.10).
    pub strings: bool,
    /// The kinds that have `bk_…`, a `fn.pure.baked` function (5.11).
    pub baked: Vec<Kind>,
    /// Whether `main` also has `$err`, unbuffered to stderr, and `$buf`, buffered to stdout and
    /// finished by a deferred `os::done` (1.13).
    pub writers: bool,
    /// The kinds that have `vsum_…`, reading a list view, and `vbump_…`, changing one (2.10).
    pub views: Vec<Kind>,
    /// `use` lines, each a path and its short name (1.11).
    pub uses: Vec<(String, String)>,
    /// A named set of failures and its handlers (8.3, 8.7).
    pub failure_family: Option<FailureFamily>,
    /// Whether `rsum`, which calls itself, and `reven` and `rodd`, which call each other, are
    /// declared (5.1).
    pub recursion: bool,
    /// A C union, declared in a shield, with its integer fields (9.6, D398).
    pub untagged: Option<(String, Vec<(String, Kind)>)>,
    pub tokens: Option<Tokens>,
    pub generics: Option<Generics>,
    pub methods: Vec<MethodDecl>,
    /// The room of an arena `main` makes, if it makes one (D330, D386).
    pub arena: Option<i128>,
    pub named: Vec<NamedType>,
    pub structs: Vec<StructDecl>,
    pub unions: Vec<UnionDecl>,
    pub procedures: Vec<Procedure>,
    pub functions: Vec<Function>,
    pub main: Vec<Statement>,
    /// Whether the printer leaves out `( )` where 3.9's table already gives the order.
    pub fewer_brackets: bool,
}

/// `fam.failure` and its `fam.handler`, with a narrowing that gives a value instead and one that
/// returns (8.3, 8.7, 8.8).
fn print_failure_family(out: &mut String, family: &FailureFamily) {
    let name = &family.name;
    let _ = writeln!(out, "fam.failure {name}:");
    for member in &family.members {
        match member.data {
            Some(kind) => {
                let _ = writeln!(out, "    {}[got: {}];", member.name, kind.word());
            }
            None => {
                let _ = writeln!(out, "    {};", member.name);
            }
        }
    }
    let _ = writeln!(out, "end fam.failure {name}\n");
    let handler = &family.handler;
    let _ = writeln!(out, "fam.handler {handler}:");
    let _ = writeln!(out, "    {handler}.fallback:");
    for (at, member) in family.members.iter().enumerate() {
        let _ = writeln!(
            out,
            "        case {}:\n            instead {at};",
            member.name
        );
    }
    let _ = writeln!(out, "    end {handler}.fallback");
    let _ = writeln!(out, "    {handler}.bail:");
    for member in &family.members {
        let _ = writeln!(
            out,
            "        case {}:\n            return void;",
            member.name
        );
    }
    let _ = writeln!(out, "    end {handler}.bail");
    let _ = writeln!(out, "end fam.handler {handler}\n");
}

/// Where a task runs: threads take no stack, and a runner that switches tasks takes the
/// allocator and size of each task's stack (D351, D356).
fn spawn_options(runner: &str) -> String {
    match runner {
        "pool" => "on: Lend $pool".to_string(),
        "events" => "on: Lend $events, alloc: Lend $heap, stack: 64 kib".to_string(),
        _ => "on: Lend $runner, alloc: Lend $heap, stack: 16 kib".to_string(),
    }
}

/// A named loop's name after `for`, and after `end for` (4.8).
fn loop_name(name: Option<&String>) -> (String, String) {
    name.map_or_else(
        || (String::new(), String::new()),
        |name| (format!("{name} "), format!(" {name}")),
    )
}

/// A `must_Give` value made, perhaps read, and handed off on every path (6.9, 6.12).
#[allow(clippy::type_complexity)] // The statement's parts, grouped as the printer reads them.
fn print_token(
    out: &mut String,
    (indent, style): (&str, Style),
    (name, struct_name, holder): (&str, &str, &str),
    (fields, ending, read_into): (&[(String, Expression)], &Ending, Option<&(String, String)>),
) {
    let built: Vec<String> = fields
        .iter()
        .map(|(field, value)| format!("{field} = {}", style.expression(value)))
        .collect();
    let built = format!("{struct_name}[{}]", built.join(", "));
    let read = |out: &mut String, from: &str| {
        if let Some((target, field)) = read_into {
            let _ = writeln!(out, "{indent}change ${target} = ${from}.{field};");
        }
    };
    match ending {
        Ending::Direct => {
            let _ = writeln!(out, "{indent}let ${name}: {struct_name} = {built};");
            read(out, name);
            let _ = writeln!(out, "{indent}tend(Give ${name});");
        }
        Ending::Deferred => {
            let _ = writeln!(out, "{indent}let ${name}: {struct_name} = {built};");
            let _ = writeln!(out, "{indent}defer tend(Give ${name});");
            read(out, name);
        }
        Ending::Branches(test) => {
            let _ = writeln!(out, "{indent}let ${name}: {struct_name} = {built};");
            read(out, name);
            let _ = writeln!(out, "{indent}if {}:", style.expression(test));
            let _ = writeln!(out, "{indent}    tend(Give ${name});");
            let _ = writeln!(out, "{indent}else:");
            let _ = writeln!(out, "{indent}    tend(Give ${name});");
            let _ = writeln!(out, "{indent}end if");
        }
        Ending::Passed { made_from, passed } => {
            let _ = writeln!(
                out,
                "{indent}let ${name}: {struct_name} = tmake({});",
                style.expression(made_from)
            );
            let _ = writeln!(
                out,
                "{indent}let ${passed}: {struct_name} = tpass(Give ${name});"
            );
            read(out, passed);
            let _ = writeln!(out, "{indent}tend(Give ${passed});");
        }
        Ending::Looped { count, item } => {
            let all = vec![built; usize::try_from(*count).unwrap_or(1)];
            let _ = writeln!(
                out,
                "{indent}let ${name}: std::coll::array<{struct_name}>[len: {count} items] = [{}];",
                all.join(", ")
            );
            let _ = writeln!(out, "{indent}for ${item} in Give ${name}:");
            if let Some((target, field)) = read_into {
                let _ = writeln!(out, "{indent}    change ${target} = ${item}.{field};");
            }
            let _ = writeln!(out, "{indent}    tend(Give ${item});");
            let _ = writeln!(out, "{indent}end for");
        }
        Ending::Matched {
            variable,
            union,
            cases,
        } => {
            let _ = writeln!(out, "{indent}let ${name}: {struct_name} = {built};");
            if let Some((target, field)) = read_into {
                let _ = writeln!(out, "{indent}change ${target} = ${name}.{field};");
            }
            let _ = writeln!(out, "{indent}match ${variable}:");
            for case in cases {
                let _ = writeln!(out, "{indent}    case {union}.{case}:");
                let _ = writeln!(out, "{indent}        tend(Give ${name});");
            }
            let _ = writeln!(out, "{indent}end match");
        }
        Ending::Held => {
            let _ = writeln!(
                out,
                "{indent}let ${name}: {holder} = {holder}[inner = {built}, n = 3];"
            );
            let _ = writeln!(out, "{indent}tclose(Give ${name});");
        }
    }
}

/// The `must_Give` struct, its holder, and the functions that make, pass and end them.
fn print_tokens(out: &mut String, tokens: &Tokens) {
    let name = &tokens.name;
    let holder = &tokens.holder;
    let _ = writeln!(out, "struct.must_Give {name}:");
    for (field, kind) in &tokens.fields {
        let _ = writeln!(out, "    {field}: {};", kind.word());
    }
    let _ = writeln!(out, "end struct.must_Give {name}\n");
    let _ = writeln!(
        out,
        "struct.must_Give {holder}:\n    inner: {name};\n    n: i64;\nend struct.must_Give {holder}\n"
    );
    let _ = writeln!(
        out,
        "fn tend [$t: Given {name}] -> void:\n    finish $t;\n    return void;\nend fn tend\n"
    );
    let first = tokens.fields.first().map_or("i32", |(_, kind)| kind.word());
    let filled: Vec<String> = tokens
        .fields
        .iter()
        .enumerate()
        .map(|(at, (field, _))| {
            if at == 0 {
                format!("{field} = $v")
            } else {
                format!("{field} = 1")
            }
        })
        .collect();
    let _ = writeln!(
        out,
        "fn tmake [$v: {first}] -> {name}:\n    return {name}[{}];\nend fn tmake\n",
        filled.join(", ")
    );
    let _ = writeln!(
        out,
        "fn tpass [$t: Given {name}] -> {name}:\n    return Give $t;\nend fn tpass\n"
    );
    let _ = writeln!(
        out,
        "fn tclose [$h: Given {holder}] -> void:\n    tend(Give $h.inner);\n    finish $h;\n    return void;\nend fn tclose\n"
    );
}

/// The interfaces, the type set, the operators and `satisfies` lines, `gbox`, and the generic
/// functions, as 5.8 and 3.10 spell them.
fn print_generics(out: &mut String, generics: &Generics) {
    out.push_str(
        "interface equatable<?self>:\n    op == [$a: Lent ?self, $b: Lent ?self] -> bool;\nend interface equatable\n\n",
    );
    out.push_str("interface ordered<?self>:\n    includes equatable;\n    op below [$a: Lent ?self, $b: Lent ?self] -> bool;\n    law if $a below $b and $b below $c then $a below $c;\nend interface ordered\n\n");
    out.push_str(
        "interface addable<?self>:\n    op + [$a: Lent ?self, $b: Lent ?self] -> ?self;\nend interface addable\n\n",
    );
    out.push_str("type_set whole:\n    i32;\n    i64;\nend type_set whole\n\n");
    let mut lines: Vec<(String, &str)> = generics
        .ordered
        .iter()
        .map(|kind| (kind.word().to_string(), "ordered"))
        .collect();
    lines.push(("i32".to_string(), "addable"));
    lines.push(("i64".to_string(), "addable"));
    if let Some(structure) = &generics.structure {
        let name = &structure.name;
        let first = structure
            .fields
            .first()
            .map_or("", |(field, _)| field.as_str());
        let _ = writeln!(
            out,
            "op below [$a: Lent {name}, $b: Lent {name}] -> bool:\n    return $a.{first} below $b.{first};\nend op below\n"
        );
        let _ = writeln!(
            out,
            "op == [$a: Lent {name}, $b: Lent {name}] -> bool:\n    return $a.{first} == $b.{first};\nend op ==\n"
        );
        // Its own `div.floor` by a number of its first field's type, dividing that field (3.10).
        let (first_field, first_kind) = structure
            .fields
            .first()
            .map_or(("", "i32"), |(field, kind)| (field.as_str(), kind.word()));
        let divided: Vec<String> = structure
            .fields
            .iter()
            .map(|(field, _)| {
                if field == first_field {
                    format!("{field} = $a.{field} div.floor $b")
                } else {
                    format!("{field} = $a.{field}")
                }
            })
            .collect();
        let _ = writeln!(
            out,
            "op div.floor [$a: Lent {name}, $b: {first_kind}] -> {name}:\n    return {name}[{}];\nend op div.floor\n",
            divided.join(", ")
        );
        // Its own `+`, `-` and `x`, field by field, wrapping so they never stop (3.10).
        for (operator, word) in [("+", "plus.wrap"), ("-", "minus.wrap"), ("x", "times.wrap")] {
            let fields: Vec<String> = structure
                .fields
                .iter()
                .map(|(field, _)| format!("{field} = $a.{field} {word} $b.{field}"))
                .collect();
            let _ = writeln!(
                out,
                "op {operator} [$a: Lent {name}, $b: Lent {name}] -> {name}:\n    return {name}[{}];\nend op {operator}\n",
                fields.join(", ")
            );
        }
        lines.push((name.clone(), "ordered"));
        lines.push((name.clone(), "addable"));
    }
    for (at, (type_name, interface)) in lines.iter().enumerate() {
        // `.|.` means exactly what `satisfies` does (5.8).
        let word = if at == generics.easter {
            ".|."
        } else {
            "satisfies"
        };
        let _ = writeln!(out, "{type_name} {word} {interface};");
    }
    out.push_str("\nstruct gbox<?item: any.type>:\n    held: ?item;\nend struct gbox\n\n");
    out.push_str("struct gpair<?left: any.type, ?right: any.type>:\n    first: ?left;\n    second: ?right;\nend struct gpair\n\n");
    out.push_str("fn gfirst<?left: any.type, ?right: any.type> [$p: Given gpair<?left, ?right>] -> ?left:\n    return Give $p.first;\nend fn gfirst\n\n");
    out.push_str("fn gmax<?item: ordered> [$a: Lent ?item, $b: Lent ?item] -> ?item:\n    if $a above $b:\n        return Copy $a;\n    end if\n    return Copy $b;\nend fn gmax\n\n");
    out.push_str("fn gsame<?item: equatable> [$a: Lent ?item, $b: Lent ?item] -> bool:\n    return $a == $b;\nend fn gsame\n\n");
    out.push_str("fn gsum<?item: whole and addable> [$a: ?item, $b: ?item] -> ?item:\n    return $a + $b;\nend fn gsum\n\n");
    out.push_str("fn gtwice<?item: addable> [$a: Lent ?item] -> ?item:\n    return $a + $a;\nend fn gtwice\n\n");
    out.push_str("fn gkeep<?item: any.type> [$a: Given ?item] -> ?item:\n    let $b: ?item = Give $a;\n    return Give $b;\nend fn gkeep\n\n");
    if generics.unit_parameter {
        out.push_str("fn glarger<?u: any.unit> [$a: f64.stable[unit: ?u], $b: f64.stable[unit: ?u]] -> f64.stable[unit: ?u]:\n    if $a above $b:\n        return $a;\n    end if\n    return $b;\nend fn glarger\n\n");
    }
}

/// The one printer: spacing as 1.14 says, one space around a binary operator and nothing inside
/// brackets. Every nested operation is grouped with `( )`, or with `fewer_brackets`, only where
/// 3.9's table doesn't already give the order, which tests the table itself.
#[allow(clippy::too_many_lines)] // Each kind of declaration in turn, then `main`.
pub fn print(program: &Program) -> String {
    let style = Style {
        fewer_brackets: program.fewer_brackets,
    };
    let mut out = String::from("import std;\nimport os;\nimport @concur;\n");
    // A C union is `@csup`'s, so the file imports it (D82, D399).
    if program.untagged.is_some() {
        out.push_str("import @csup;\n");
    }
    out.push('\n');
    for (path, short) in &program.uses {
        let _ = writeln!(out, "use {path} as {short};");
    }
    if !program.uses.is_empty() {
        out.push('\n');
    }
    for family in &program.unit_families {
        let _ = writeln!(out, "{} {}:", family.keyword, family.name);
        for (unit, definition) in &family.units {
            let _ = writeln!(out, "    {unit}{definition};");
        }
        let _ = writeln!(out, "end {} {}\n", family.keyword, family.name);
    }
    let mut families: Vec<&String> = Vec::new();
    for named in &program.named {
        match &named.family {
            None => {
                let word = if named.alias {
                    "type.alias"
                } else {
                    "type.new"
                };
                let _ = writeln!(out, "{word} {} = {};", named.name, named.kind.word());
            }
            Some(family) if !families.contains(&family) => {
                families.push(family);
                // A family states its kind once (2.18).
                let word = if named.alias {
                    "fam.type.alias"
                } else {
                    "fam.type.new"
                };
                let _ = writeln!(out, "{word} {family}:");
                for member in program
                    .named
                    .iter()
                    .filter(|other| other.family.as_ref() == Some(family))
                {
                    let _ = writeln!(out, "    {} = {};", member.used(), member.kind.word());
                }
                let _ = writeln!(out, "end {word} {family}");
            }
            Some(_) => {}
        }
    }
    // Names for a struct, a bounded list and a bounded string (2.18).
    for (name, alias, written) in &program.named_others {
        let word = if *alias { "type.alias" } else { "type.new" };
        let _ = writeln!(out, "{word} {name} = {written};");
    }
    if !program.named.is_empty() || !program.named_others.is_empty() {
        out.push('\n');
    }
    for declared in &program.unions {
        let _ = writeln!(out, "union.tagged {}:", declared.name);
        for (case, fields) in &declared.cases {
            if fields.is_empty() {
                let _ = writeln!(out, "    {case};");
            } else {
                let fields: Vec<String> = fields
                    .iter()
                    .map(|(field, kind)| format!("{field}: {}", kind.word()))
                    .collect();
                let _ = writeln!(out, "    {case}[{}];", fields.join(", "));
            }
        }
        let _ = writeln!(out, "end union.tagged {}\n", declared.name);
        // A method on a union, its `match` returning from every arm (5.6, 4.3).
        if declared.sized {
            let name = &declared.name;
            let _ = writeln!(
                out,
                "fn {name}.%size [$u: Lent {name}] -> i32:\n    match $u:"
            );
            for (at, (case, fields)) in declared.cases.iter().enumerate() {
                match fields.iter().find(|(_, kind)| *kind == Kind::I32) {
                    Some((field, _)) => {
                        let _ = writeln!(
                            out,
                            "        case {name}.{case} $c:\n            return $c.{field};"
                        );
                    }
                    None => {
                        let _ =
                            writeln!(out, "        case {name}.{case}:\n            return {at};");
                    }
                }
            }
            let _ = writeln!(out, "    end match\nend fn {name}.%size\n");
            // A function a union is lent to (7.5).
            let _ = writeln!(
                out,
                "fn uwidth_{name} [$u: Lent {name}] -> i32:\n    return $u.%size();\nend fn uwidth_{name}\n"
            );
        }
    }
    for declared in &program.structs {
        let _ = writeln!(out, "{} {}:", declared.layout.word(), declared.name);
        for (field, kind) in &declared.fields {
            let _ = writeln!(
                out,
                "    {field}: {};",
                declared.field_type(field, *kind).word()
            );
        }
        let _ = writeln!(out, "end {} {}\n", declared.layout.word(), declared.name);
    }
    if let Some(generics) = &program.generics {
        print_generics(&mut out, generics);
    }
    if let Some((inner, kind)) = &program.nest {
        let _ = writeln!(
            out,
            "struct nest:\n    inner: {inner};\n    count: i32;\n    items: std::coll::list<{}>[len.max: 3 items];\nend struct nest\n",
            kind.word()
        );
        let _ = writeln!(
            out,
            "struct view <`src`>:\n    of: Lend from `src` {inner};\nend struct view\n"
        );
    }
    if let Some((name, field)) = &program.warm {
        let _ = writeln!(
            out,
            "fn rwarm [$a: Lent {name}, $b: Lent {name}] -> Lend from <$a, $b> {name}:\n    if $a.{field} above $b.{field}:\n        return Lend $a;\n    end if\n    return Lend $b;\nend fn rwarm\n"
        );
    }
    for kind in &program.trios {
        let kind = kind.word();
        let _ = writeln!(
            out,
            "fn trio_{kind} [$n: {kind}] -> std::coll::array<{kind}>[len: 3 items]:\n    return [$n, $n, $n];\nend fn trio_{kind}\n"
        );
    }
    for kind in &program.baked {
        let kind = kind.word();
        let _ = writeln!(
            out,
            "fn.pure.baked bk_{kind} [$n: {kind}] -> {kind}:\n    return ($n times.wrap 3) plus.wrap 1;\nend fn.pure.baked bk_{kind}\n"
        );
    }
    for kind in &program.views {
        let kind = kind.word();
        let _ = writeln!(
            out,
            "fn vsum_{kind} [$xs: std::coll::list<{kind}>] -> {kind}:\n    let mut $t: {kind} = 0;\n    for $x in Lend $xs:\n        change $t = $t plus.wrap $x;\n    end for\n    return $t;\nend fn vsum_{kind}\n"
        );
        let _ = writeln!(
            out,
            "fn vbump_{kind} [$xs: std::coll::list.mut<{kind}>] -> void:\n    for $x in Lend.mut $xs:\n        change $x = $x plus.wrap 1;\n    end for\n    return void;\nend fn vbump_{kind}\n"
        );
    }
    if let Some(family) = &program.failure_family {
        print_failure_family(&mut out, family);
    }
    if program.strings {
        out.push_str("struct tag:\n    name: str;\n    n: i32;\nend struct tag\n\n");
        out.push_str("fn tpick [$a: str, $b: str, $first: bool] -> Lend from <$a, $b> str:\n    if $first:\n        return Lend $a;\n    end if\n    return Lend $b;\nend fn tpick\n\n");
        out.push_str(
            "fn tcount [$a: str, $n: i32] -> i32:\n    return $n plus.wrap 1;\nend fn tcount\n\n",
        );
    }
    if program.recursion {
        out.push_str("fn rsum [$n: u64] -> u64:\n    if $n == 0:\n        return 0;\n    end if\n    return $n plus.wrap rsum($n - 1);\nend fn rsum\n\n");
        out.push_str("fn reven [$n: u64] -> u64:\n    if $n == 0:\n        return 1;\n    end if\n    return rodd($n - 1);\nend fn reven\n\n");
        out.push_str("fn rodd [$n: u64] -> u64:\n    if $n == 0:\n        return 0;\n    end if\n    return reven($n - 1);\nend fn rodd\n\n");
    }
    if let Some(tokens) = &program.tokens {
        print_tokens(&mut out, tokens);
    }
    if let Some((union, fields)) = &program.untagged {
        let _ = writeln!(
            out,
            "shield c_{union}:\n    reason \"a generated program's C union\"\n    union.@csup::untagged {union}:"
        );
        for (field, kind) in fields {
            let _ = writeln!(out, "        {field}: {};", kind.word());
        }
        let _ = writeln!(
            out,
            "    end union.@csup::untagged {union}\nend shield c_{union}\n"
        );
    }
    for function in &program.functions {
        let parameters: Vec<String> = function
            .parameters
            .iter()
            .map(|(name, value_type)| format!("${name}: {}", value_type.word()))
            .collect();
        let parameters = if parameters.is_empty() {
            "none".to_string()
        } else {
            parameters.join(", ")
        };
        let fails = if function.failures.is_empty() {
            String::new()
        } else {
            // A named set is listed by its name, once, for all its members (8.1, 8.3).
            let mut listed: Vec<String> = Vec::new();
            for failure in &function.failures {
                let written = match (&failure.family, failure.data) {
                    (Some(family), _) => family.clone(),
                    (None, Some(kind)) => format!("{}[got: {}]", failure.name, kind.word()),
                    (None, None) => failure.name.clone(),
                };
                if !listed.contains(&written) {
                    listed.push(written);
                }
            }
            format!(" fails [{}]", listed.join(", "))
        };
        let word = if function.pure { "fn.pure" } else { "fn" };
        let _ = writeln!(
            out,
            "{word} {} [{parameters}] -> {}{fails}:",
            function.name,
            function.result.word()
        );
        // `labels` and `defaults` go first in the body, in that order (5.4).
        if !function.labels.is_empty() {
            let labels: Vec<String> = function
                .labels
                .iter()
                .map(|(parameter, label)| format!("${parameter} as {label}"))
                .collect();
            let word = if function.any_order {
                "labels.any_order"
            } else {
                "labels"
            };
            let _ = writeln!(out, "    {word} [{}];", labels.join(", "));
        }
        if !function.defaults.is_empty() {
            let defaults: Vec<String> = function
                .defaults
                .iter()
                .map(|(parameter, value)| format!("${parameter} = {}", style.expression(value)))
                .collect();
            let _ = writeln!(out, "    defaults [{}];", defaults.join(", "));
        }
        // Then `requires` and `ensures`, before the code (5.4).
        if let Some(test) = &function.requires {
            let _ = writeln!(out, "    requires {};", style.expression(test));
        }
        if let Some((comparison, value)) = &function.ensures {
            let _ = writeln!(
                out,
                "    ensures result {} {};",
                comparison.word(),
                style.grouped(value)
            );
        }
        print_block(&mut out, &function.body, 1, style);
        let _ = writeln!(out, "    return {};", style.expression(&function.returned));
        let _ = writeln!(out, "end {word} {}\n", function.name);
    }
    for method in &program.methods {
        let structure = &method.struct_name;
        match &method.changes {
            None => {
                let _ = writeln!(
                    out,
                    "fn {structure}.%{} [$r: Lent {structure}] -> {}:",
                    method.name,
                    method.kind.word()
                );
                let _ = writeln!(out, "    return {};", style.expression(&method.value));
                let _ = writeln!(out, "end fn {structure}.%{}\n", method.name);
            }
            Some(field) => {
                let _ = writeln!(
                    out,
                    "fn {structure}.%%{} [$r: Lent.mut {structure}, $by: {}] -> void:",
                    method.name,
                    method.kind.word()
                );
                // `labels` goes first in the body (5.4).
                if method.labelled {
                    out.push_str("    labels [$by as by];\n");
                }
                let _ = writeln!(
                    out,
                    "    change $r.{field} = {};",
                    style.expression(&method.value)
                );
                out.push_str("    return void;\n");
                let _ = writeln!(out, "end fn {structure}.%%{}\n", method.name);
            }
        }
    }
    for procedure in &program.procedures {
        let parameters: Vec<String> = procedure
            .parameters
            .iter()
            .map(|(name, kind)| match kind {
                ParamKind::Changed(kind) => format!("${name}: Lent.mut {}", kind.word()),
                ParamKind::Plain(kind) => format!("${name}: {}", kind.word()),
                ParamKind::Record { struct_name, given } => {
                    // How a struct is held is a choice, so it's always written (D393).
                    let word = if *given { "Given " } else { "Lent " };
                    format!("${name}: {word}{struct_name}")
                }
            })
            .collect();
        let _ = writeln!(
            out,
            "fn {} [{}] -> void:",
            procedure.name,
            parameters.join(", ")
        );
        print_block(&mut out, &procedure.body, 1, style);
        out.push_str("    return void;\n");
        let _ = writeln!(out, "end fn {}\n", procedure.name);
    }
    out.push_str("fn.main [none] -> void:\n");
    out.push_str("    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();\n");
    out.push_str("    let $runner: std::run::one_thread = std::run::one_thread();\n");
    out.push_str("    let $heap: os::heap = os::heap();\n");
    if program.runners {
        out.push_str("    let $pool: os::threads = os::threads(count: 2);\n");
        out.push_str("    defer os::done(Give $pool);\n");
        out.push_str("    let $events: os::events = os::events(count: 1, alloc: Lend $heap);\n");
        out.push_str("    defer os::done(Give $events);\n");
    }
    if program.writers {
        out.push_str("    let mut $err: os::stderr.unbuffered = os::stderr.unbuffered();\n");
        out.push_str(
            "    let mut $buf: os::stdout.buffered = os::stdout.buffered(room: 1024 bytes);\n",
        );
        out.push_str("    defer os::done(Give $buf);\n");
    }
    if let Some(room) = program.arena {
        let _ = writeln!(
            out,
            "    let $arena: std::mem::arena = std::mem::arena.new(room: {room} bytes, alloc: Lend $heap);"
        );
        out.push_str("    defer std::mem::arena.done(Give $arena, alloc: Lend $heap);\n");
    }
    print_block(&mut out, &program.main, 1, style);
    out.push_str("    return void;\nend fn.main\n");
    out
}

/// How the printer groups.
#[derive(Clone, Copy)]
pub struct Style {
    pub fewer_brackets: bool,
}

#[allow(clippy::too_many_lines)] // One arm per kind of statement, read top to bottom.
fn print_block(out: &mut String, statements: &[Statement], depth: usize, style: Style) {
    let indent = "    ".repeat(depth);
    for statement in statements {
        match statement {
            Statement::Let {
                name,
                value_type,
                value,
            } => {
                let _ = writeln!(
                    out,
                    "{indent}let mut ${name}: {} = {};",
                    value_type.word(),
                    style.expression(value)
                );
            }
            Statement::Change { name, value } => {
                let _ = writeln!(out, "{indent}change ${name} = {};", style.expression(value));
            }
            Statement::LetArray {
                name,
                kind,
                len,
                filling,
            } => {
                let filling = match filling {
                    Filling::Items(items) => {
                        let items: Vec<String> =
                            items.iter().map(|item| style.expression(item)).collect();
                        format!("[{}]", items.join(", "))
                    }
                    Filling::All(value) => format!("all {}", style.grouped(value)),
                };
                let _ = writeln!(
                    out,
                    "{indent}let mut ${name}: std::coll::array<{}>[len: {len} items] = {filling};",
                    kind.word()
                );
            }
            Statement::LetList {
                name,
                kind,
                max,
                items,
                allocator,
                grow,
            } => {
                let shown = if items.is_empty() {
                    "empty".to_string()
                } else {
                    let items: Vec<String> =
                        items.iter().map(|item| style.expression(item)).collect();
                    format!("[{}]", items.join(", "))
                };
                let kind = kind.word();
                match allocator {
                    None => {
                        let _ = writeln!(
                            out,
                            "{indent}let mut ${name}: std::coll::list<{kind}>[len.max: {max} items] = {shown};"
                        );
                    }
                    Some(allocator) => {
                        let value = if items.is_empty() {
                            shown
                        } else {
                            format!("std::coll::list.from({shown}, alloc: Lend ${allocator})")
                        };
                        let grow = grow
                            .as_ref()
                            .map_or_else(String::new, |grow| format!(", grow: {grow}"));
                        let _ = writeln!(
                            out,
                            "{indent}let mut ${name}: std::coll::list<{kind}>[len.max: inf items{grow}] = {value};"
                        );
                        let _ = writeln!(
                            out,
                            "{indent}defer std::coll::list.done(Give ${name}, alloc: Lend ${allocator});"
                        );
                    }
                }
            }
            Statement::Append {
                handled,
                list,
                value,
                allocator,
            } => {
                let allocator = allocator
                    .as_ref()
                    .map_or_else(String::new, |found| format!(", alloc: Lend ${found}"));
                let handled = if *handled { " ?? void" } else { "" };
                let _ = writeln!(
                    out,
                    "{indent}${list}.std::%%append(val: {}{allocator}){handled};",
                    style.expression(value)
                );
            }
            Statement::HeapText { name, from } => {
                let _ = writeln!(
                    out,
                    "{indent}let mut ${name}: str[len.max: inf bytes] = str.from({}, alloc: Lend $heap);",
                    style.expression(from)
                );
                let _ = writeln!(
                    out,
                    "{indent}defer str.done(Give ${name}, alloc: Lend $heap);"
                );
                let _ = writeln!(
                    out,
                    "{indent}std::io::fmt_print(Lend.mut $out, \"{{${name}}}{{newln}}\");"
                );
            }
            Statement::Heap => {
                let _ = writeln!(out, "{indent}let $heap: os::heap = os::heap();");
            }
            Statement::Call(call) => {
                let _ = writeln!(out, "{indent}{};", style.expression(call));
            }
            Statement::Split {
                name,
                kind,
                list,
                at_end,
            } => {
                let at = if *at_end {
                    format!("${list}.len")
                } else {
                    "0".to_string()
                };
                let _ = writeln!(
                    out,
                    "{indent}let ${name}: std::coll::halves<{}> = ${list}.std::%split(at.items: {at});",
                    kind.word()
                );
            }
            Statement::LetStruct {
                name,
                struct_name,
                fields,
            } => {
                let fields: Vec<String> = fields
                    .iter()
                    .map(|(field, value)| format!("{field} = {}", style.expression(value)))
                    .collect();
                let _ = writeln!(
                    out,
                    "{indent}let mut ${name}: {struct_name} = {struct_name}[{}];",
                    fields.join(", ")
                );
            }
            Statement::ChangeField {
                variable,
                field,
                value,
            } => {
                let _ = writeln!(
                    out,
                    "{indent}change ${variable}.{field} = {};",
                    style.expression(value)
                );
            }
            Statement::ChangeItem {
                array,
                position,
                value,
            } => {
                let _ = writeln!(
                    out,
                    "{indent}change {} = {};",
                    item(array, position),
                    style.expression(value)
                );
            }
            Statement::Print(value) => {
                let _ = writeln!(
                    out,
                    "{indent}std::io::fmt_print(Lend.mut $out, \"{{}}{{newln}}\", {});",
                    style.expression(value)
                );
            }
            Statement::Format {
                writer,
                stored,
                text,
                values,
            } => {
                let mut shown = String::new();
                for value in values {
                    let _ = write!(shown, ", {}", style.expression(value));
                }
                let values = shown;
                let stored = if *stored { ".stored" } else { "" };
                let _ = writeln!(
                    out,
                    "{indent}std::io::fmt_print{stored}(Lend.mut ${writer}, \"{text}{{newln}}\"{values});"
                );
            }
            Statement::Flush => {
                let _ = writeln!(out, "{indent}$buf.%%flush();");
            }
            Statement::If {
                test,
                then,
                otherwise,
            } => {
                let _ = writeln!(out, "{indent}if {}:", style.expression(test));
                print_block(out, then, depth + 1, style);
                if !otherwise.is_empty() {
                    let _ = writeln!(out, "{indent}else:");
                    print_block(out, otherwise, depth + 1, style);
                }
                let _ = writeln!(out, "{indent}end if");
            }
            Statement::While {
                counter,
                rounds,
                body,
            } => {
                let _ = writeln!(out, "{indent}let mut ${counter}: i32 = 0;");
                let _ = writeln!(out, "{indent}while ${counter} below {rounds}:");
                // The count goes up first, so a `break` or nothing else can skip it.
                let _ = writeln!(out, "{indent}    change ${counter} = ${counter} + 1;");
                print_block(out, body, depth + 1, style);
                let _ = writeln!(out, "{indent}end while");
            }
            Statement::For {
                name,
                counter,
                from,
                end,
                inclusive,
                step,
                body,
            } => {
                let label = if *inclusive { "to" } else { "before" };
                let step = step.map_or_else(String::new, |step| format!(", step_by: {step}"));
                let (opener, closer) = loop_name(name.as_ref());
                let _ = writeln!(
                    out,
                    "{indent}for {opener}${counter} in range[from: {from}, {label}: {}{step}]:",
                    style.expression(end)
                );
                print_block(out, body, depth + 1, style);
                let _ = writeln!(out, "{indent}end for{closer}");
            }
            Statement::Break => {
                let _ = writeln!(out, "{indent}break;");
            }
            Statement::BreakNamed(name) => {
                let _ = writeln!(out, "{indent}break {name};");
            }
            Statement::ForEach {
                name,
                item,
                array,
                changes,
                body,
            } => {
                // A loop says how it holds each item, even to read it (D393).
                let word = if *changes { "Lend.mut " } else { "Lend " };
                let (opener, closer) = loop_name(name.as_ref());
                let _ = writeln!(out, "{indent}for {opener}${item} in {word}${array}:");
                print_block(out, body, depth + 1, style);
                let _ = writeln!(out, "{indent}end for{closer}");
            }
            Statement::Decl {
                name,
                mutable,
                kind,
            } => {
                let word = if *mutable { "decl mut" } else { "decl" };
                let _ = writeln!(out, "{indent}{word} ${name}: {};", kind.word());
            }
            Statement::Assign { name, value } => {
                let _ = writeln!(out, "{indent}assign ${name} = {};", style.expression(value));
            }
            Statement::Sequence(statements) => print_block(out, statements, depth, style),
            Statement::DeclAssign {
                name,
                mutable,
                kind,
                test,
                then_value,
                else_value,
            } => {
                let word = if *mutable { "decl mut" } else { "decl" };
                let _ = writeln!(out, "{indent}{word} ${name}: {};", kind.word());
                let _ = writeln!(out, "{indent}if {}:", style.expression(test));
                let _ = writeln!(
                    out,
                    "{indent}    assign ${name} = {};",
                    style.expression(then_value)
                );
                let _ = writeln!(out, "{indent}else:");
                let _ = writeln!(
                    out,
                    "{indent}    assign ${name} = {};",
                    style.expression(else_value)
                );
                let _ = writeln!(out, "{indent}end if");
            }
            Statement::Const {
                name,
                value_type,
                value,
            } => {
                let _ = writeln!(
                    out,
                    "{indent}const ${name}: {} = {};",
                    value_type.word(),
                    style.expression(value)
                );
            }
            Statement::LetCase {
                name,
                union_name,
                case,
                fields,
            } => {
                let value = if fields.is_empty() {
                    format!("{union_name}.{case}")
                } else {
                    let fields: Vec<String> = fields
                        .iter()
                        .map(|(field, value)| format!("{field} = {}", style.expression(value)))
                        .collect();
                    format!("{union_name}.{case}[{}]", fields.join(", "))
                };
                let _ = writeln!(out, "{indent}let mut ${name}: {union_name} = {value};");
            }
            Statement::Match {
                variable,
                union_name,
                arms,
            } => {
                let _ = writeln!(out, "{indent}match ${variable}:");
                for (case, named, body) in arms {
                    let named = named
                        .as_ref()
                        .map_or_else(String::new, |name| format!(" ${name}"));
                    let _ = writeln!(out, "{indent}    case {union_name}.{case}{named}:");
                    print_block(out, body, depth + 2, style);
                }
                let _ = writeln!(out, "{indent}end match");
            }
            Statement::Task {
                task,
                name,
                kind,
                call,
                runner,
            } => {
                let kind = kind.word();
                let _ = writeln!(
                    out,
                    "{indent}let ${task}: @concur::task<{kind}> = @concur::spawn({}) {};",
                    spawn_options(runner),
                    style.expression(call)
                );
                let _ = writeln!(
                    out,
                    "{indent}let mut ${name}: {kind} = @concur::task.done(Give ${task});"
                );
            }
            Statement::WrapStruct {
                wrapped,
                new_type,
                word,
                source,
                back,
                struct_name,
            } => {
                let _ = writeln!(
                    out,
                    "{indent}let mut ${wrapped}: {new_type} = {word} ${source} as {new_type};"
                );
                let _ = writeln!(
                    out,
                    "{indent}let mut ${back}: {struct_name} = Give ${wrapped} as {struct_name};"
                );
            }
            Statement::LetNamed {
                name,
                written,
                value,
                back,
            } => {
                let _ = writeln!(
                    out,
                    "{indent}let mut ${name}: {written} = {};",
                    style.expression(value)
                );
                if let Some((back, kind)) = back {
                    let kind = kind.word();
                    let _ = writeln!(out, "{indent}let mut ${back}: {kind} = ${name} as {kind};");
                }
            }
            Statement::Reveal {
                shield,
                target,
                secret,
            } => {
                let _ = writeln!(out, "{indent}shield {shield}:");
                let _ = writeln!(
                    out,
                    "{indent}    reason \"a generated program shows its secrets\""
                );
                let _ = writeln!(out, "{indent}    change ${target} = ${secret}.%reveal();");
                let _ = writeln!(out, "{indent}end shield {shield}");
            }
            Statement::Untagged {
                shield,
                name,
                union,
                written: (field, value),
                read,
                read_into,
            } => {
                let _ = writeln!(out, "{indent}shield {shield}:");
                let _ = writeln!(
                    out,
                    "{indent}    reason \"a generated program reads a C union's bytes\""
                );
                let _ = writeln!(
                    out,
                    "{indent}    let ${name}: union.@csup::untagged {union} = {union}[{field} = {}, rest = all 0];",
                    style.expression(value)
                );
                let _ = writeln!(out, "{indent}    change ${read_into} = ${name}.{read};");
                let _ = writeln!(out, "{indent}end shield {shield}");
            }
            Statement::Token {
                name,
                struct_name,
                holder,
                fields,
                ending,
                read_into,
            } => print_token(
                out,
                (&indent, style),
                (name, struct_name, holder),
                (fields, ending, read_into.as_ref()),
            ),
            Statement::BorrowStruct {
                how,
                view,
                records: (record, other),
                struct_name,
                field,
                target,
                declare,
                value,
            } => {
                let into = match declare {
                    Some(kind) => format!("let ${target}: {} =", kind.word()),
                    None => format!("change ${target} ="),
                };
                match *how {
                    "held" => {
                        let _ = writeln!(out, "{indent}if true:");
                        let _ = writeln!(
                        out,
                        "{indent}    let ${view}: Lend.held from ${record} {struct_name} = Lend ${record};"
                    );
                        let _ = writeln!(out, "{indent}    {into} ${view}.{field};");
                        let _ = writeln!(out, "{indent}end if");
                    }
                    "mut" => {
                        let _ = writeln!(
                        out,
                        "{indent}let ${view}: Lend.mut from ${record} {struct_name} = Lend.mut ${record};"
                    );
                        let _ = writeln!(
                            out,
                            "{indent}change ${view}.{field} = {};",
                            style.expression(value)
                        );
                    }
                    "returned" => {
                        let _ = writeln!(
                        out,
                        "{indent}let ${view}: Lend from <${record}, ${other}> {struct_name} = rwarm(Lend ${record}, Lend ${other});"
                    );
                        let _ = writeln!(out, "{indent}{into} ${view}.{field};");
                    }
                    _ => {
                        let _ = writeln!(
                        out,
                        "{indent}let ${view}: Lend from ${record} {struct_name} = Lend ${record};"
                    );
                        let _ = writeln!(out, "{indent}{into} ${view}.{field};");
                    }
                }
            }
            Statement::Defer(statement) => {
                let mut inner = String::new();
                print_block(
                    &mut inner,
                    std::slice::from_ref(statement.as_ref()),
                    0,
                    style,
                );
                let _ = write!(out, "{indent}defer {inner}");
            }
            Statement::FreshLoop { item, call, target } => {
                let _ = writeln!(out, "{indent}for ${item} in {}:", style.expression(call));
                let _ = writeln!(
                    out,
                    "{indent}    change ${target} = ${target} plus.wrap ${item};"
                );
                let _ = writeln!(out, "{indent}end for");
            }
            Statement::SplitTasks {
                halves,
                list,
                kind,
                at_end,
                tasks: (left, right),
                runner,
                target,
            } => {
                let kind = kind.word();
                let at = if *at_end {
                    format!("${list}.len")
                } else {
                    "0".to_string()
                };
                let _ = writeln!(
                    out,
                    "{indent}let ${halves}: std::coll::halves<{kind}> = ${list}.std::%split(at.items: {at});"
                );
                for (task, half) in [(left, "first"), (right, "second")] {
                    let _ = writeln!(
                        out,
                        "{indent}let ${task}: @concur::task<{kind}> = @concur::spawn({}) vsum_{kind}(${halves}.{half});",
                        spawn_options(runner)
                    );
                }
                let _ = writeln!(
                    out,
                    "{indent}change ${target} = @concur::task.done(Give ${left}) plus.wrap @concur::task.done(Give ${right});"
                );
            }
            Statement::Atomic {
                name,
                kind,
                numbers: (start, add, expect),
                old,
                now,
                acquire,
            } => {
                let kind = kind.word();
                let _ = writeln!(
                    out,
                    "{indent}let ${name}: @concur::atomic<{kind}> = @concur::atomic.new({start});"
                );
                let _ = writeln!(out, "{indent}${name}.%add.wrap(val: {add});");
                let _ = writeln!(
                    out,
                    "{indent}let ${old}: {kind} = ${name}.%swap(val: {expect});"
                );
                let _ = writeln!(
                    out,
                    "{indent}${name}.%compare_swap(expect: {expect}, val: {add}) otherwise:"
                );
                let _ = writeln!(out, "{indent}    case not_expected $f:");
                let _ = writeln!(out, "{indent}        ${name}.%add.wrap(val: $f.val);");
                let _ = writeln!(out, "{indent}end otherwise");
                let ordering = if *acquire { ".acquire" } else { "" };
                let _ = writeln!(
                    out,
                    "{indent}let ${now}: {kind} = ${name}.%load{ordering}();"
                );
            }
            Statement::Locks {
                locks,
                bound,
                kind,
                target,
            } => {
                let kind = kind.word();
                for (lock, start) in locks {
                    let _ = writeln!(
                        out,
                        "{indent}let mut ${lock}: @concur::lock<{kind}> = @concur::lock.new({start});"
                    );
                }
                let names: Vec<String> = locks.iter().map(|(lock, _)| format!("${lock}")).collect();
                let as_names: Vec<String> = bound.iter().map(|name| format!("${name}")).collect();
                let _ = writeln!(
                    out,
                    "{indent}@concur::locked {} as {}:",
                    names.join(", "),
                    as_names.join(", ")
                );
                for name in bound {
                    let _ = writeln!(out, "{indent}    change ${name} = ${name} plus.wrap 1;");
                }
                if let Some(first) = bound.first() {
                    let _ = writeln!(out, "{indent}    change ${target} = ${first};");
                }
                let _ = writeln!(out, "{indent}end @concur::locked");
            }
            Statement::Chunks {
                array,
                lanes,
                mode,
                chunk,
                leftover,
                target,
            } => {
                let how = match *mode {
                    "pad" | "mut.pad" => format!("{mode}(lanes: {lanes}, pad: 0)"),
                    other => format!("{other}(lanes: {lanes})"),
                };
                // `.mut` writes each chunk back, so it's `%%`, and changes the chunk (2.21).
                let marks = if mode.starts_with("mut") { "%%" } else { "%" };
                let _ = writeln!(
                    out,
                    "{indent}for ${chunk} in ${array}.std::{marks}simd_chunks.{how}:"
                );
                if mode.starts_with("mut") {
                    let _ = writeln!(
                        out,
                        "{indent}    change ${chunk} = ${chunk} plus.wrap ${chunk};"
                    );
                } else {
                    let _ = writeln!(
                        out,
                        "{indent}    change ${target} = ${target} plus.wrap ${chunk}.%max_val();"
                    );
                }
                if *mode == "leftovers" {
                    let _ = writeln!(out, "{indent}leftovers ${leftover}:");
                    let _ = writeln!(
                        out,
                        "{indent}    change ${target} = ${target} plus.wrap ${leftover};"
                    );
                }
                let _ = writeln!(out, "{indent}end for");
            }
            Statement::Map {
                name,
                kind,
                key,
                value,
            } => {
                let _ = writeln!(
                    out,
                    "{indent}let ${name}: std::coll::map.{kind}<key.type: {key}, val.type: {}> = empty;",
                    value.word()
                );
                let _ = writeln!(
                    out,
                    "{indent}defer std::coll::map.done(Give ${name}, alloc: Lend $heap);"
                );
            }
            Statement::PickText {
                name,
                left,
                right,
                test,
            } => {
                let _ = writeln!(
                    out,
                    "{indent}let ${name}: Lend from <${left}, ${right}> str = tpick(${left}, ${right}, {});",
                    style.expression(test)
                );
                let _ = writeln!(
                    out,
                    "{indent}std::io::fmt_print(Lend.mut $out, \"{{${name}}}{{newln}}\");"
                );
            }
            Statement::LetStructValue {
                name,
                struct_name,
                value,
            } => {
                let _ = writeln!(
                    out,
                    "{indent}let mut ${name}: {struct_name} = {};",
                    style.expression(value)
                );
            }
            Statement::GenericPair {
                name,
                left,
                right,
                first,
                second,
                read_into,
            } => {
                let _ = writeln!(
                    out,
                    "{indent}let ${name}: gpair<{}, {right}> = gpair[first = {}, second = {}];",
                    left.word(),
                    style.expression(first),
                    style.expression(second)
                );
                let _ = writeln!(out, "{indent}change ${read_into} = gfirst(Give ${name});");
            }
            Statement::GenericBox {
                name,
                kind,
                value,
                read_into,
            } => {
                let kind = kind.word();
                let _ = writeln!(
                    out,
                    "{indent}let ${name}: gbox<{kind}> = gbox[held = {}];",
                    style.expression(value)
                );
                let _ = writeln!(out, "{indent}change ${read_into} = ${name}.held;");
            }
            Statement::Assembly {
                shield,
                result,
                instruction,
                reads,
                otherwise,
            } => {
                let reads: Vec<String> = reads
                    .iter()
                    .enumerate()
                    .map(|(register, name)| format!("x{register} = ${name}"))
                    .collect();
                let _ = writeln!(out, "{indent}on.target aarch64:");
                let _ = writeln!(out, "{indent}    shield {shield}:");
                let _ = writeln!(
                    out,
                    "{indent}        reason \"a generated program's one instruction\""
                );
                let _ = writeln!(out, "{indent}        inline.asm:");
                let _ = writeln!(out, "{indent}            reads: [{}];", reads.join(", "));
                let _ = writeln!(out, "{indent}            writes: [x0 => ${result}];");
                let _ = writeln!(out, "{indent}            clobbers: [none];");
                let _ = writeln!(out, "{indent}            {instruction}");
                let _ = writeln!(out, "{indent}        end inline.asm");
                let _ = writeln!(out, "{indent}    end shield {shield}");
                let _ = writeln!(out, "{indent}else:");
                let _ = writeln!(
                    out,
                    "{indent}    let ${result}: i64 = {};",
                    style.expression(otherwise)
                );
                let _ = writeln!(out, "{indent}end on.target");
            }
            Statement::Pointer {
                shield,
                family,
                writes,
                kind,
                pointer,
                target,
                moves,
                read_into,
                mutable,
            } => {
                let inner = format!("{indent}    ");
                let (word, lend) = if *mutable {
                    (".mut", "Lend.mut")
                } else {
                    ("", "Lend")
                };
                let pointer_type = format!("ptr.{family}{word}<{}>", kind.word());
                let _ = writeln!(out, "{indent}shield {shield}:");
                let _ = writeln!(
                    out,
                    "{inner}reason \"a generated program goes through a pointer\""
                );
                let _ = writeln!(
                    out,
                    "{inner}let ${pointer}: {pointer_type} = {pointer_type}.of({lend} ${target});"
                );
                if let Some(value) = writes {
                    let _ = writeln!(out, "{inner}${pointer}.%%write(val: {value});");
                }
                let mut last = pointer.clone();
                if let Some((count, there, back)) = moves {
                    let bytes = count * kind.bytes();
                    let _ = writeln!(
                        out,
                        "{inner}let ${there}: {pointer_type} = ${pointer}.%moved(offset.items: {count});"
                    );
                    let _ = writeln!(
                        out,
                        "{inner}let ${back}: {pointer_type} = ${there}.%moved(offset.bytes: {});",
                        -bytes
                    );
                    last.clone_from(back);
                }
                match read_into {
                    Some(variable) => {
                        let _ = writeln!(out, "{inner}change ${variable} = ${last}.%read();");
                    }
                    None => {
                        let _ = writeln!(
                            out,
                            "{inner}std::io::fmt_print(Lend.mut $out, \"{{}}{{newln}}\", ${last}.%read());"
                        );
                    }
                }
                let _ = writeln!(out, "{indent}end shield {shield}");
            }
            Statement::Test {
                variable,
                written,
                body,
            } => {
                let _ = writeln!(out, "{indent}if ${variable}: {written}:");
                print_block(out, body, depth + 1, style);
                let _ = writeln!(out, "{indent}end if");
            }
            Statement::CallChanging {
                receiver,
                name,
                value,
                labelled,
            } => {
                let label = if *labelled { "by: " } else { "" };
                let _ = writeln!(
                    out,
                    "{indent}${receiver}.%%{name}({label}{});",
                    style.expression(value)
                );
            }
            Statement::CallProcedure { name, arguments } => {
                let _ = writeln!(out, "{indent}{name}({});", arguments.join(", "));
            }
            Statement::Forever {
                counter,
                rounds,
                body,
            } => {
                let _ = writeln!(out, "{indent}let mut ${counter}: i32 = 0;");
                let _ = writeln!(out, "{indent}forever:");
                let _ = writeln!(out, "{indent}    change ${counter} = ${counter} + 1;");
                let _ = writeln!(out, "{indent}    if ${counter} above {rounds}:");
                let _ = writeln!(out, "{indent}        break;");
                let _ = writeln!(out, "{indent}    end if");
                print_block(out, body, depth + 1, style);
                let _ = writeln!(out, "{indent}end forever");
            }
            Statement::Fail { name, data } => match data {
                Some(value) => {
                    let _ = writeln!(
                        out,
                        "{indent}fail {name}[got = {}];",
                        style.expression(value)
                    );
                }
                None => {
                    let _ = writeln!(out, "{indent}fail {name};");
                }
            },
            Statement::LetOtherwise {
                name,
                value_type,
                call,
                arms,
                handler,
            } => {
                if let Some(handler) = handler {
                    let _ = writeln!(
                        out,
                        "{indent}let mut ${name}: {} = {} otherwise {handler};",
                        value_type.word(),
                        style.expression(call)
                    );
                    continue;
                }
                let _ = writeln!(
                    out,
                    "{indent}let mut ${name}: {} = {} otherwise:",
                    value_type.word(),
                    style.expression(call)
                );
                for (failure, named, arm) in arms {
                    match named {
                        Some(named) => {
                            let _ = writeln!(out, "{indent}    case {failure} ${named}:");
                        }
                        None => {
                            let _ = writeln!(out, "{indent}    case {failure}:");
                        }
                    }
                    let done = match arm {
                        Arm::Instead(value) => format!("instead {}", style.expression(value)),
                        Arm::Return(Some(value)) => format!("return {}", style.expression(value)),
                        Arm::Return(None) => "return void".to_string(),
                        Arm::Break => "break".to_string(),
                        Arm::Fail(failure, Some(value)) => {
                            format!("fail {failure}[got = {}]", style.expression(value))
                        }
                        Arm::Fail(failure, None) => format!("fail {failure}"),
                    };
                    let _ = writeln!(out, "{indent}        {done};");
                }
                let _ = writeln!(out, "{indent}end otherwise");
            }
        }
    }
}

impl Style {
    /// An expression, with a nested operation grouped where it has to be.
    #[allow(clippy::too_many_lines)] // One arm per kind of expression.
    pub fn expression(self, value: &Expression) -> String {
        match value {
            Expression::Integer(number) => number.to_string(),
            Expression::Float(text) => text.clone(),
            Expression::As { variable, target } => format!("${variable} as {}", target.word()),
            Expression::AsNamed { variable, written } => format!("${variable} as {written}"),
            Expression::AsOrElse { value, instead } => format!(
                "({}) ?? {}",
                self.expression(value),
                self.expression(instead)
            ),
            Expression::WithUnit { number, unit } => format!("{number} {unit}"),
            Expression::FloatWithUnit { number, unit } => format!("{number} {unit}"),
            Expression::Convert {
                variable,
                narrowing,
                target,
            } => format!("${variable} as{narrowing} {}", target.word()),
            Expression::Lanes(lanes) => {
                let lanes: Vec<String> = lanes.iter().map(|lane| self.expression(lane)).collect();
                format!("[{}]", lanes.join(", "))
            }
            Expression::AllLanes(value) => format!("all {}", self.grouped(value)),
            Expression::Pick {
                test,
                if_true,
                if_false,
            } => format!(
                "{}.%pick(if_true: {}, if_false: {})",
                self.grouped(test),
                self.expression(if_true),
                self.expression(if_false)
            ),
            Expression::NotBit(inner) => format!("not.bit {}", self.grouped(inner)),
            Expression::SimdMethod {
                receiver,
                method,
                argument,
            } => match argument {
                Some((label, value)) => {
                    format!("${receiver}.%{method}({label}: {})", self.expression(value))
                }
                None => format!("${receiver}.%{method}()"),
            },
            Expression::Shuffle {
                receiver,
                lanes,
                from_zero,
            } => {
                let label = if *from_zero { "offset.lanes" } else { "pos" };
                let lanes: Vec<String> = lanes.iter().map(ToString::to_string).collect();
                format!("${receiver}.%shuffle({label}: [{}])", lanes.join(", "))
            }
            Expression::Method { receiver, method } => {
                format!("{}.%{method}()", self.grouped(receiver))
            }
            Expression::Text(text) if text.is_empty() => "empty".to_string(),
            Expression::Text(text) => {
                format!("\"{}\"", text.replace('\\', "\\\\").replace('"', "\\\""))
            }
            Expression::Bool(truth) => truth.to_string(),
            Expression::Variable(name) => format!("${name}"),
            Expression::Binary {
                operator,
                left,
                right,
            } => {
                let (left, right) = match operator.level() {
                    Some(level) if self.fewer_brackets => {
                        (self.side(left, level, false), self.side(right, level, true))
                    }
                    _ => (self.grouped(left), self.grouped(right)),
                };
                format!("{left} {} {right}", operator.word())
            }
            Expression::Compare {
                comparison,
                left,
                right,
            } => {
                let (left, right) = if self.fewer_brackets {
                    (self.side(left, 5, false), self.side(right, 5, true))
                } else {
                    (self.grouped(left), self.grouped(right))
                };
                format!("{left} {} {right}", comparison.word())
            }
            Expression::Chain {
                values,
                comparisons,
            } => {
                let mut text = self.side(&values[0], 5, true);
                for (comparison, value) in comparisons.iter().zip(&values[1..]) {
                    let _ = write!(text, " {} {}", comparison.word(), self.side(value, 5, true));
                }
                text
            }
            Expression::Logic {
                and,
                both,
                left,
                right,
            } => {
                let level = if *and { 7 } else { 8 };
                // The table doesn't list the `.both` forms, so their sides are always grouped.
                let (left, right) = if self.fewer_brackets && !both {
                    (self.side(left, level, false), self.side(right, level, true))
                } else {
                    (self.grouped(left), self.grouped(right))
                };
                let word = match (and, both) {
                    (true, false) => "and",
                    (true, true) => "and.both",
                    (false, false) => "or",
                    (false, true) => "or.both",
                };
                format!("{left} {word} {right}")
            }
            Expression::Not(inner) => {
                let inner = if self.fewer_brackets {
                    self.side(inner, 6, false)
                } else {
                    self.grouped(inner)
                };
                format!("not {inner}")
            }
            Expression::Negate(inner) => format!("-{}", self.grouped(inner)),
            Expression::Call {
                function,
                arguments,
                handling,
            } => {
                let arguments: Vec<String> = arguments
                    .iter()
                    .map(|(label, argument)| match label {
                        Some(label) => format!("{label}: {}", self.expression(argument)),
                        None => self.expression(argument),
                    })
                    .collect();
                let call = format!("{function}({})", arguments.join(", "));
                match handling {
                    Handling::Crash => call,
                    Handling::Pass => format!("{call}?"),
                    Handling::Fallback(value) => format!("{call} ?? {}", self.grouped(value)),
                }
            }
            Expression::Item { array, position } => item(array, position),
            Expression::Field { variable, field } => format!("${variable}.{field}"),
            Expression::Lend(variable) => format!("Lend ${variable}"),
            Expression::Bake(call) => format!("bake {}", self.expression(call)),
            Expression::Build {
                struct_name,
                fields,
            } => {
                let fields: Vec<String> = fields
                    .iter()
                    .map(|(field, value)| format!("{field} = {}", self.expression(value)))
                    .collect();
                format!("{struct_name}[{}]", fields.join(", "))
            }
            Expression::LendMut(variable) => format!("Lend.mut ${variable}"),
            Expression::CallMethod { receiver, name } => format!("${receiver}.%{name}()"),
        }
    }

    /// A single value as it is, anything else in `( )`.
    pub fn grouped(self, value: &Expression) -> String {
        match value {
            // `??` binds as tightly as a call (3.9), but its value is grouped anyway, so no
            // reading of that is relied on unless brackets are left out.
            Expression::Call {
                handling: Handling::Fallback(_),
                ..
            } => format!("({})", self.expression(value)),
            Expression::Variable(_)
            | Expression::Text(_)
            | Expression::Bool(_)
            | Expression::Call { .. }
            | Expression::Item { .. }
            | Expression::Method { .. }
            | Expression::Pick { .. }
            | Expression::CallMethod { .. }
            | Expression::Field { .. } => self.expression(value),
            Expression::Integer(number) if *number >= 0 => self.expression(value),
            Expression::Float(text) if !text.starts_with('-') && !text.starts_with("~-") => {
                self.expression(value)
            }
            _ => format!("({})", self.expression(value)),
        }
    }

    /// One side of an operation at `level`: left bare when it binds tighter, or as tight on the
    /// left, since operations at one level go left to right (3.9), and grouped otherwise.
    fn side(self, value: &Expression, level: u8, right: bool) -> String {
        let inner = match value {
            Expression::Binary { operator, .. } => operator.level(),
            Expression::Compare { .. } => Some(5),
            Expression::Not(_) => Some(6),
            Expression::Logic {
                and, both: false, ..
            } => Some(if *and { 7 } else { 8 }),
            // A chain is grouped wherever it sits, so it's never read as a longer chain, and so
            // is a `.both` form, which the table doesn't list.
            Expression::Chain { .. } | Expression::Logic { .. } => None,
            _ => return self.grouped(value),
        };
        match inner {
            Some(inner) if inner < level || (inner == level && !right && level != 5) => {
                self.expression(value)
            }
            _ => format!("({})", self.expression(value)),
        }
    }
}

/// An item, as `$a[pos: 2]`, `$a[offset.items: 1]` or `$a[pos: $v3]`.
fn item(array: &str, position: &Position) -> String {
    match position {
        Position::Pos(number) => format!("${array}[pos: {number}]"),
        Position::Offset(number) => format!("${array}[offset.items: {number}]"),
        Position::PosOf(name) => format!("${array}[pos: ${name}]"),
    }
}
