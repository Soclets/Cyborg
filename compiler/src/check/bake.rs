//! Compile-time work (D91): `const` values, `bake`, and the `fn.pure.baked` functions they call, run on the checked
//! program by a small interpreter. It always finishes, through a step limit that refuses the work (D89, D218), and
//! a check that fails while baking refuses the program with the same message.

use std::collections::HashMap;

use crate::checked::{
    Arithmetic, Block, CheckFailure, Comparison, Expression, ExpressionKind, FloatArithmetic, MachineType, Program,
    Statement, WhenPart,
};
use crate::diagnostic::{Diagnostic, Kind};
use crate::source::Span;

/// How many statements compile-time work may run, the oracle's limit too (standard 12.1).
const STEP_LIMIT: u64 = 10_000_000;

/// A value known while compiling.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Constant {
    Integer(i128),
    Float(f64),
    Bool(bool),
}

impl Constant {
    /// The value as the checked program writes it.
    #[must_use]
    pub fn expression(self, machine_type: MachineType) -> Expression {
        let kind = match self {
            Constant::Integer(value) => ExpressionKind::Integer(value),
            Constant::Float(value) => ExpressionKind::Float(value),
            Constant::Bool(value) => ExpressionKind::Bool(value),
        };
        Expression { kind, machine_type }
    }
}

/// Why compile-time work stopped.
#[derive(Debug)]
pub enum Stop {
    /// A check failed, so the program is refused with its message.
    Refused(Box<Diagnostic>),
    /// The step limit was reached in the loop at this span.
    StepLimit(Span),
    /// Something the interpreter doesn't run, which is the compiler's own gap.
    NotBuilt(String),
}

enum Flow {
    Normal,
    Return(Option<Constant>),
    Break(usize),
}

pub struct Interpreter<'a> {
    program: &'a Program,
    steps: u64,
    /// Locals of the code being baked that are known, like `const`s in the body around it.
    pub known: HashMap<usize, Constant>,
    /// The loop being run, for the step limit's error.
    loop_span: Option<Span>,
}

impl<'a> Interpreter<'a> {
    #[must_use]
    pub fn new(program: &'a Program) -> Interpreter<'a> {
        Interpreter { program, steps: 0, known: HashMap::new(), loop_span: None }
    }

    /// Works out an expression in the body around it, whose known locals are in `known`.
    ///
    /// # Errors
    /// Why it stopped.
    pub fn evaluate(&mut self, expression: &Expression) -> Result<Constant, Stop> {
        let mut locals: HashMap<usize, Constant> = self.known.clone();
        self.expression(expression, &mut locals)
    }

    fn step(&mut self) -> Result<(), Stop> {
        self.steps += 1;
        if self.steps > STEP_LIMIT {
            return Err(Stop::StepLimit(self.loop_span.unwrap_or_default()));
        }
        Ok(())
    }

    fn block(&mut self, block: &Block, locals: &mut HashMap<usize, Constant>) -> Result<Flow, Stop> {
        for statement in block {
            self.step()?;
            match self.statement(statement, locals)? {
                Flow::Normal => {}
                other => return Ok(other),
            }
        }
        Ok(Flow::Normal)
    }

    fn statement(&mut self, statement: &Statement, locals: &mut HashMap<usize, Constant>) -> Result<Flow, Stop> {
        match statement {
            Statement::Let { local, value } | Statement::Set { local, value } => {
                let value = self.expression(value, locals)?;
                locals.insert(*local, value);
                Ok(Flow::Normal)
            }
            Statement::Evaluate(value) => {
                self.expression(value, locals)?;
                Ok(Flow::Normal)
            }
            Statement::If { condition, then, otherwise } => {
                if self.boolean(condition, locals)? {
                    self.block(then, locals)
                } else {
                    self.block(otherwise, locals)
                }
            }
            Statement::Loop { id, body, opener } => {
                let outer = self.loop_span.replace(*opener);
                loop {
                    self.step()?;
                    match self.block(body, locals)? {
                        Flow::Normal => {}
                        Flow::Break(target) if target == *id => break,
                        other => {
                            self.loop_span = outer;
                            return Ok(other);
                        }
                    }
                }
                self.loop_span = outer;
                Ok(Flow::Normal)
            }
            Statement::Break(id) => Ok(Flow::Break(*id)),
            Statement::Flow(_) => Ok(Flow::Normal),
            Statement::Return(value) => match value {
                Some(value) => Ok(Flow::Return(Some(self.expression(value, locals)?))),
                None => Ok(Flow::Return(None)),
            },
            Statement::Check { condition, failure, operands } => {
                let values =
                    operands.iter().map(|operand| self.expression(operand, locals)).collect::<Result<Vec<_>, _>>()?;
                if !self.boolean(condition, locals)? {
                    let types: Vec<MachineType> = operands.iter().map(|operand| operand.machine_type.clone()).collect();
                    return Err(refuse(failure, &values, &types));
                }
                Ok(Flow::Normal)
            }
            _ => Err(Stop::NotBuilt("this statement while baking".to_string())),
        }
    }

    fn boolean(&mut self, expression: &Expression, locals: &mut HashMap<usize, Constant>) -> Result<bool, Stop> {
        match self.expression(expression, locals)? {
            Constant::Bool(value) => Ok(value),
            _ => Err(Stop::NotBuilt("a condition that isn't a bool while baking".to_string())),
        }
    }

    #[allow(clippy::too_many_lines)]
    fn expression(&mut self, expression: &Expression, locals: &mut HashMap<usize, Constant>) -> Result<Constant, Stop> {
        let machine_type = &expression.machine_type;
        match &expression.kind {
            ExpressionKind::Integer(value) => Ok(Constant::Integer(*value)),
            ExpressionKind::Float(value) => Ok(Constant::Float(*value)),
            ExpressionKind::Bool(value) => Ok(Constant::Bool(*value)),
            ExpressionKind::Local(local) => {
                locals.get(local).copied().ok_or_else(|| Stop::NotBuilt("a value not known while baking".to_string()))
            }
            ExpressionKind::Arithmetic { operator, left, right, failure, more } => {
                let left_value = self.expression(left, locals)?;
                let right_value = self.expression(right, locals)?;
                integer_arithmetic(*operator, left_value, right_value, machine_type, failure.as_deref(), more)
            }
            ExpressionKind::FloatArithmetic { operator, left, right, infinity, nan } => {
                let (Constant::Float(a), Constant::Float(b)) =
                    (self.expression(left, locals)?, self.expression(right, locals)?)
                else {
                    return Err(Stop::NotBuilt("float maths on something else while baking".to_string()));
                };
                let bits = if let MachineType::Float { bits } = machine_type { *bits } else { 64 };
                let result = match operator {
                    FloatArithmetic::Add => a + b,
                    FloatArithmetic::Subtract => a - b,
                    FloatArithmetic::Multiply => a * b,
                    FloatArithmetic::Divide => a / b,
                };
                #[allow(clippy::cast_possible_truncation)]
                let result = if bits == 32 { f64::from(result as f32) } else { result };
                let types = [MachineType::Float { bits }, MachineType::Float { bits }];
                let values = [Constant::Float(a), Constant::Float(b)];
                if result.is_infinite()
                    && let Some(failure) = infinity
                {
                    return Err(refuse(failure, &values, &types));
                }
                if result.is_nan()
                    && let Some(failure) = nan
                {
                    return Err(refuse(failure, &values, &types));
                }
                Ok(Constant::Float(result))
            }
            ExpressionKind::Negate { operand, failure } => match self.expression(operand, locals)? {
                Constant::Integer(value) => {
                    let (bits, signed) = integer_shape(machine_type);
                    let (lowest, _) = range(bits, signed);
                    if signed
                        && value == lowest
                        && let Some(failure) = failure
                    {
                        return Err(refuse(failure, &[Constant::Integer(value)], &[machine_type.clone()]));
                    }
                    Ok(Constant::Integer(wrap(-value, bits, signed)))
                }
                Constant::Float(value) => Ok(Constant::Float(-value)),
                Constant::Bool(_) => Err(Stop::NotBuilt("negating a bool while baking".to_string())),
            },
            ExpressionKind::Compare { operator, left, right } => {
                let left_value = self.expression(left, locals)?;
                let right_value = self.expression(right, locals)?;
                let ordering = match (left_value, right_value) {
                    (Constant::Integer(a), Constant::Integer(b)) => a.partial_cmp(&b),
                    (Constant::Float(a), Constant::Float(b)) => a.partial_cmp(&b),
                    (Constant::Bool(a), Constant::Bool(b)) => a.partial_cmp(&b),
                    _ => None,
                };
                let result = match (operator, ordering) {
                    (Comparison::NotEqual, None) => true,
                    (_, None) => false,
                    (Comparison::Equal, Some(order)) => order.is_eq(),
                    (Comparison::NotEqual, Some(order)) => order.is_ne(),
                    (Comparison::Below, Some(order)) => order.is_lt(),
                    (Comparison::Above, Some(order)) => order.is_gt(),
                    (Comparison::AtLeast, Some(order)) => order.is_ge(),
                    (Comparison::AtMost, Some(order)) => order.is_le(),
                };
                Ok(Constant::Bool(result))
            }
            ExpressionKind::Not(operand) => Ok(Constant::Bool(!self.boolean(operand, locals)?)),
            ExpressionKind::BitNot(operand) => match self.expression(operand, locals)? {
                Constant::Integer(value) => {
                    let (bits, signed) = integer_shape(machine_type);
                    Ok(Constant::Integer(wrap(!value, bits, signed)))
                }
                Constant::Bool(value) => Ok(Constant::Bool(!value)),
                Constant::Float(_) => Err(Stop::NotBuilt("`bit_not` on a float while baking".to_string())),
            },
            ExpressionKind::And(left, right) => {
                Ok(Constant::Bool(self.boolean(left, locals)? && self.boolean(right, locals)?))
            }
            ExpressionKind::Or(left, right) => {
                Ok(Constant::Bool(self.boolean(left, locals)? || self.boolean(right, locals)?))
            }
            ExpressionKind::Convert(operand) => match self.expression(operand, locals)? {
                Constant::Integer(value) => {
                    let (bits, signed) = integer_shape(machine_type);
                    Ok(Constant::Integer(wrap(value, bits, signed)))
                }
                other => Ok(other),
            },
            ExpressionKind::Fit { value, min, max, failure } => {
                let checked = self.expression(value, locals)?;
                if let Constant::Integer(number) = checked
                    && (number < *min || number > *max)
                {
                    return Err(refuse(failure, &[checked], &[value.machine_type.clone()]));
                }
                Ok(checked)
            }
            ExpressionKind::FitFloat { value, min, max, failure } => {
                let checked = self.expression(value, locals)?;
                if let Constant::Float(number) = checked
                    && !(number >= *min && number <= *max)
                {
                    return Err(refuse(failure, &[checked], &[value.machine_type.clone()]));
                }
                Ok(checked)
            }
            ExpressionKind::Bind { local, value, body } => {
                let value = self.expression(value, locals)?;
                locals.insert(*local, value);
                self.expression(body, locals)
            }
            ExpressionKind::Call { function, arguments } => {
                let mut values = Vec::new();
                for argument in arguments {
                    let crate::checked::Argument::Value(value) = argument else {
                        return Err(Stop::NotBuilt("an argument that isn't a plain value while baking".to_string()));
                    };
                    values.push(self.expression(value, locals)?);
                }
                self.call(*function, &values)
            }
            _ => Err(Stop::NotBuilt("this kind of value while baking".to_string())),
        }
    }

    fn call(&mut self, function: usize, arguments: &[Constant]) -> Result<Constant, Stop> {
        let program = self.program;
        let Some(callee) = program.functions.get(function) else {
            return Err(Stop::NotBuilt("a call to a missing function while baking".to_string()));
        };
        let mut locals = HashMap::new();
        for (parameter, value) in callee.parameters.iter().zip(arguments) {
            locals.insert(parameter.local, *value);
        }
        match self.block(&callee.body, &mut locals)? {
            Flow::Return(Some(value)) => Ok(value),
            _ => Err(Stop::NotBuilt("a baked function that gives nothing back".to_string())),
        }
    }
}

fn integer_shape(machine_type: &MachineType) -> (u32, bool) {
    match machine_type {
        MachineType::Integer { bits, signed } => (*bits, *signed),
        MachineType::Bool => (1, false),
        _ => (64, true),
    }
}

fn range(bits: u32, signed: bool) -> (i128, i128) {
    if bits >= 128 {
        return (i128::MIN, i128::MAX);
    }
    if signed { (-(1i128 << (bits - 1)), (1i128 << (bits - 1)) - 1) } else { (0, (1i128 << bits) - 1) }
}

/// A value cut to a width, the way the machine holds it.
fn wrap(value: i128, bits: u32, signed: bool) -> i128 {
    if bits >= 128 {
        return value;
    }
    let mask = (1i128 << bits) - 1;
    let low = value & mask;
    if signed && low >= (1i128 << (bits - 1)) { low - (1i128 << bits) } else { low }
}

/// Integer arithmetic while baking, with the same checks as while running (standard 10.4).
fn integer_arithmetic(
    operator: Arithmetic,
    left: Constant,
    right: Constant,
    machine_type: &MachineType,
    failure: Option<&CheckFailure>,
    more: &[CheckFailure],
) -> Result<Constant, Stop> {
    let (Constant::Integer(a), Constant::Integer(b)) = (left, right) else {
        if let (Constant::Bool(a), Constant::Bool(b)) = (left, right) {
            return Ok(Constant::Bool(match operator {
                Arithmetic::BitAnd => a & b,
                _ => a | b,
            }));
        }
        return Err(Stop::NotBuilt("maths on something else while baking".to_string()));
    };
    let (bits, signed) = integer_shape(machine_type);
    let (lowest, highest) = range(bits, signed);
    let values = [left, right];
    let types = [machine_type.clone(), machine_type.clone()];
    let fails = |failure: Option<&CheckFailure>| -> Stop {
        match failure {
            Some(failure) => refuse(failure, &values, &types),
            None => Stop::NotBuilt("a check with no message while baking".to_string()),
        }
    };
    let result = match operator {
        Arithmetic::Add | Arithmetic::Subtract | Arithmetic::Multiply => {
            let exact = match operator {
                Arithmetic::Add => a + b,
                Arithmetic::Subtract => a - b,
                _ => a.saturating_mul(b),
            };
            if (exact < lowest || exact > highest) && failure.is_some() {
                return Err(fails(failure));
            }
            wrap(exact, bits, signed)
        }
        Arithmetic::WrappingAdd => wrap(a + b, bits, signed),
        Arithmetic::DivideTruncate
        | Arithmetic::DivideFloor
        | Arithmetic::DivideExact
        | Arithmetic::RemainderTruncate
        | Arithmetic::RemainderFloor => {
            if b == 0 {
                return Err(fails(failure));
            }
            if signed && a == lowest && b == -1 {
                return Err(fails(more.last()));
            }
            let quotient = a / b;
            let remainder = a % b;
            let floors = remainder != 0 && ((remainder < 0) != (b < 0));
            match operator {
                Arithmetic::DivideTruncate => quotient,
                Arithmetic::RemainderTruncate => remainder,
                Arithmetic::DivideExact => {
                    if remainder != 0 {
                        return Err(fails(more.first()));
                    }
                    quotient
                }
                Arithmetic::DivideFloor => {
                    if floors {
                        quotient - 1
                    } else {
                        quotient
                    }
                }
                _ => {
                    if floors {
                        remainder + b
                    } else {
                        remainder
                    }
                }
            }
        }
        Arithmetic::BitAnd => a & b,
        Arithmetic::BitOr => a | b,
        Arithmetic::BitXor => a ^ b,
        Arithmetic::ShiftLeft | Arithmetic::ShiftRight => {
            if b < 0 || b >= i128::from(bits) {
                return Err(fails(more.first()));
            }
            let shift = u32::try_from(b).unwrap_or(0);
            if operator == Arithmetic::ShiftLeft {
                let shifted = wrap(a << shift, bits, signed);
                if (shifted >> shift) != a {
                    return Err(fails(failure));
                }
                shifted
            } else {
                if signed && a < 0 {
                    return Err(fails(failure));
                }
                a >> shift
            }
        }
    };
    Ok(Constant::Integer(result))
}

/// A check that failed while baking: its message, refused, with the values filled in.
fn refuse(failure: &CheckFailure, values: &[Constant], types: &[MachineType]) -> Stop {
    let Some(source) = &failure.source else {
        return Stop::NotBuilt("a check with no message while baking".to_string());
    };
    let mut diagnostic = (**source).clone();
    if diagnostic.kind == Kind::CheckFailed {
        diagnostic.kind = Kind::Refused;
    }
    if !failure.when_template.is_empty() {
        let mut when = String::new();
        for part in &failure.when_template {
            match part {
                WhenPart::Text(text) => when.push_str(text),
                WhenPart::Operand(index) => {
                    let shown = match (values.get(*index), types.get(*index)) {
                        (Some(Constant::Integer(value)), _) => value.to_string(),
                        (Some(Constant::Float(value)), Some(MachineType::Float { bits })) => {
                            super::floats::shortest(*value, *bits)
                        }
                        (Some(Constant::Float(value)), _) => super::floats::shortest(*value, 64),
                        (Some(Constant::Bool(value)), _) => value.to_string(),
                        (None, _) => String::new(),
                    };
                    when.push_str(&shown);
                }
            }
        }
        diagnostic.when = Some(when);
    }
    Stop::Refused(Box::new(diagnostic))
}
