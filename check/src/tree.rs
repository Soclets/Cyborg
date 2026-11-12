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
}

impl Type {
    pub fn word(self) -> &'static str {
        match self {
            Type::Integer(kind) => kind.word(),
            Type::Bool => "bool",
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

    pub fn is_shift(self) -> bool {
        matches!(self, Operator::ShiftLeft | Operator::ShiftRight)
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
    Call {
        function: String,
        arguments: Vec<Expression>,
    },
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
}

#[derive(Clone, Debug)]
pub struct Function {
    pub name: String,
    pub parameters: Vec<(String, Type)>,
    pub result: Type,
    pub body: Vec<Statement>,
    pub returned: Expression,
}

#[derive(Clone, Debug)]
pub struct Program {
    pub functions: Vec<Function>,
    pub main: Vec<Statement>,
}

/// The one printer: spacing as 1.14 says, one space around a binary operator, nothing inside
/// brackets, and every nested binary grouped with `( )`, so no precedence reading is relied on.
pub fn print(program: &Program) -> String {
    let mut out = String::from("import std;\nimport os;\n\n");
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
        let _ = writeln!(
            out,
            "fn {} [{parameters}] -> {}:",
            function.name,
            function.result.word()
        );
        print_block(&mut out, &function.body, 1);
        let _ = writeln!(out, "    return {};", expression(&function.returned));
        let _ = writeln!(out, "end fn {}\n", function.name);
    }
    out.push_str("fn.main [none] -> void:\n");
    out.push_str("    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();\n");
    print_block(&mut out, &program.main, 1);
    out.push_str("    return void;\nend fn.main\n");
    out
}

fn print_block(out: &mut String, statements: &[Statement], depth: usize) {
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
                    expression(value)
                );
            }
            Statement::Change { name, value } => {
                let _ = writeln!(out, "{indent}change ${name} = {};", expression(value));
            }
            Statement::Print(value) => {
                let _ = writeln!(
                    out,
                    "{indent}std::io::fmt_print(Lend.mut $out, \"{{}}{{newln}}\", {});",
                    expression(value)
                );
            }
            Statement::If {
                test,
                then,
                otherwise,
            } => {
                let _ = writeln!(out, "{indent}if {}:", expression(test));
                print_block(out, then, depth + 1);
                if !otherwise.is_empty() {
                    let _ = writeln!(out, "{indent}else:");
                    print_block(out, otherwise, depth + 1);
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
                print_block(out, body, depth + 1);
                let _ = writeln!(out, "{indent}    change ${counter} = ${counter} + 1;");
                let _ = writeln!(out, "{indent}end while");
            }
        }
    }
}

/// An expression, with a nested operation grouped.
pub fn expression(value: &Expression) -> String {
    match value {
        Expression::Integer(number) => number.to_string(),
        Expression::Bool(truth) => truth.to_string(),
        Expression::Variable(name) => format!("${name}"),
        Expression::Binary {
            operator,
            left,
            right,
        } => format!("{} {} {}", grouped(left), operator.word(), grouped(right)),
        Expression::Compare {
            comparison,
            left,
            right,
        } => format!("{} {} {}", grouped(left), comparison.word(), grouped(right)),
        Expression::Logic { and, left, right } => format!(
            "{} {} {}",
            grouped(left),
            if *and { "and" } else { "or" },
            grouped(right)
        ),
        Expression::Not(inner) => format!("not {}", grouped(inner)),
        Expression::Negate(inner) => format!("-{}", grouped(inner)),
        Expression::Call {
            function,
            arguments,
        } => {
            let arguments: Vec<String> = arguments.iter().map(expression).collect();
            format!("{function}({})", arguments.join(", "))
        }
    }
}

/// A part of a larger expression: a single value as it is, anything else in `( )`.
fn grouped(value: &Expression) -> String {
    match value {
        Expression::Variable(_) | Expression::Bool(_) | Expression::Call { .. } => {
            expression(value)
        }
        Expression::Integer(number) if *number >= 0 => expression(value),
        _ => format!("({})", expression(value)),
    }
}
