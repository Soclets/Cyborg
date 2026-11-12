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
}

impl Type {
    pub fn word(self) -> String {
        match self {
            Type::Integer(kind) => kind.word().to_string(),
            Type::Bool => "bool".to_string(),
            Type::Float(true) => "f64.stable".to_string(),
            Type::Float(false) => "f32.stable".to_string(),
            Type::Bounded(kind, low, high) => format!("{}[min: {low}, max: {high}]", kind.word()),
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
    Bool(bool),
    Variable(String),
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
        arguments: Vec<Expression>,
        handling: Handling,
    },
    /// An array's item, `$a[pos: 2]` or `$a[offset.items: 1]` (2.17).
    Item {
        array: String,
        position: Position,
    },
    /// A struct's field, `$r.f3` (7.3).
    Field {
        variable: String,
        field: String,
    },
}

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

/// A tagged union, each case with integer data or none (7.5).
#[derive(Clone, Debug)]
pub struct UnionDecl {
    pub name: String,
    pub cases: Vec<(String, Vec<(String, Kind)>)>,
}

/// A struct with integer fields (7.1).
#[derive(Clone, Debug)]
pub struct StructDecl {
    pub name: String,
    pub layout: Layout,
    pub fields: Vec<(String, Kind)>,
}

/// A failure a function lists, `bad3` or `bad4[got: i32]` (8.1).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FailureDecl {
    pub name: String,
    pub data: Option<Kind>,
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
    /// `change $a[pos: 2] = …;` (2.17)
    ChangeItem {
        array: String,
        position: Position,
        value: Expression,
    },
    Print(Expression),
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
        counter: String,
        from: i128,
        end: i128,
        /// `to:` when true, `before:` when false.
        inclusive: bool,
        step: Option<i128>,
        body: Vec<Statement>,
    },
    /// `break;`, which stops the innermost loop (4.8).
    Break,
    /// `for $x in $a:`, reading each item, or `for $x in Lend.mut $a:`, changing each (4.4).
    ForEach {
        item: String,
        array: String,
        changes: bool,
        body: Vec<Statement>,
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
        arms: Vec<(String, Expression)>,
    },
}

#[derive(Clone, Debug)]
pub struct Function {
    pub name: String,
    pub parameters: Vec<(String, Type)>,
    pub result: Type,
    pub failures: Vec<FailureDecl>,
    pub body: Vec<Statement>,
    pub returned: Expression,
}

#[derive(Clone, Debug)]
pub struct Program {
    pub structs: Vec<StructDecl>,
    pub unions: Vec<UnionDecl>,
    pub functions: Vec<Function>,
    pub main: Vec<Statement>,
    /// Whether the printer leaves out `( )` where 3.9's table already gives the order.
    pub fewer_brackets: bool,
}

/// The one printer: spacing as 1.14 says, one space around a binary operator and nothing inside
/// brackets. Every nested operation is grouped with `( )`, or with `fewer_brackets`, only where
/// 3.9's table doesn't already give the order, which tests the table itself.
pub fn print(program: &Program) -> String {
    let style = Style {
        fewer_brackets: program.fewer_brackets,
    };
    let mut out = String::from("import std;\nimport os;\n\n");
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
    }
    for declared in &program.structs {
        let _ = writeln!(out, "{} {}:", declared.layout.word(), declared.name);
        for (field, kind) in &declared.fields {
            let _ = writeln!(out, "    {field}: {};", kind.word());
        }
        let _ = writeln!(out, "end {} {}\n", declared.layout.word(), declared.name);
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
            let listed: Vec<String> = function
                .failures
                .iter()
                .map(|failure| match failure.data {
                    Some(kind) => format!("{}[got: {}]", failure.name, kind.word()),
                    None => failure.name.clone(),
                })
                .collect();
            format!(" fails [{}]", listed.join(", "))
        };
        let _ = writeln!(
            out,
            "fn {} [{parameters}] -> {}{fails}:",
            function.name,
            function.result.word()
        );
        print_block(&mut out, &function.body, 1, style);
        let _ = writeln!(out, "    return {};", style.expression(&function.returned));
        let _ = writeln!(out, "end fn {}\n", function.name);
    }
    out.push_str("fn.main [none] -> void:\n");
    out.push_str("    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();\n");
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
                counter,
                from,
                end,
                inclusive,
                step,
                body,
            } => {
                let label = if *inclusive { "to" } else { "before" };
                let step = step.map_or_else(String::new, |step| format!(", step_by: {step}"));
                let _ = writeln!(
                    out,
                    "{indent}for ${counter} in range[from: {from}, {label}: {end}{step}]:"
                );
                print_block(out, body, depth + 1, style);
                let _ = writeln!(out, "{indent}end for");
            }
            Statement::Break => {
                let _ = writeln!(out, "{indent}break;");
            }
            Statement::ForEach {
                item,
                array,
                changes,
                body,
            } => {
                let word = if *changes { "Lend.mut " } else { "" };
                let _ = writeln!(out, "{indent}for ${item} in {word}${array}:");
                print_block(out, body, depth + 1, style);
                let _ = writeln!(out, "{indent}end for");
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
            } => {
                let _ = writeln!(
                    out,
                    "{indent}let mut ${name}: {} = {} otherwise:",
                    value_type.word(),
                    style.expression(call)
                );
                for (failure, value) in arms {
                    let _ = writeln!(out, "{indent}    case {failure}:");
                    let _ = writeln!(out, "{indent}        instead {};", style.expression(value));
                }
                let _ = writeln!(out, "{indent}end otherwise");
            }
        }
    }
}

impl Style {
    /// An expression, with a nested operation grouped where it has to be.
    pub fn expression(self, value: &Expression) -> String {
        match value {
            Expression::Integer(number) => number.to_string(),
            Expression::Float(text) => text.clone(),
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
            Expression::Logic { and, left, right } => {
                let level = if *and { 7 } else { 8 };
                let (left, right) = if self.fewer_brackets {
                    (self.side(left, level, false), self.side(right, level, true))
                } else {
                    (self.grouped(left), self.grouped(right))
                };
                format!("{left} {} {right}", if *and { "and" } else { "or" })
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
                    .map(|argument| self.expression(argument))
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
            | Expression::Bool(_)
            | Expression::Call { .. }
            | Expression::Item { .. }
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
            Expression::Logic { and, .. } => Some(if *and { 7 } else { 8 }),
            // A chain is grouped wherever it sits, so it's never read as a longer chain.
            Expression::Chain { .. } => None,
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
