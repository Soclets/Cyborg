//! Expressions: their types, what they compute, and the runtime checks that stay with them (standard 10.4).

use crate::checked::{self, Arithmetic, Comparison, ExpressionKind as Kind, MachineType};
use crate::diagnostic::{Diagnostic, Kind as DiagnosticKind};
use crate::source::Span;
use crate::syntax::{self, BinaryOperator, ExpressionKind, UnaryOperator};

use super::types::{Bounds, IntegerKind, Type};
use super::{Body, Check, Checker, Origin, Reported, State};

pub const CONVERT_WHY: &str = "`:` doesn't convert, and `as` builds a new value of another type (D7)";
const NOT_DECIDED: &str = "not decided yet, and the standard doesn't say";
const NOT_DECIDED_OPERATORS: &str = "not decided yet (standard 12.3)";

/// A checked value.
#[derive(Clone, Debug)]
pub struct Typed {
    pub value_type: Type,
    pub value: checked::Expression,
    /// The number, if this is an integer literal, so where it lands can check it fits while building (D20).
    pub literal: Option<i128>,
    /// The variable, if this reads one plainly, so errors can name it.
    pub variable: Option<String>,
}

impl Typed {
    #[must_use]
    pub fn new(value_type: Type, kind: Kind) -> Typed {
        let machine_type = value_type.machine();
        Typed { value_type, value: checked::Expression { kind, machine_type }, literal: None, variable: None }
    }
}

/// Where a runtime value goes in a failure's `when:` line.
pub enum When {
    Text(String),
    /// The checked operation's operand: 0 is the left one, or the value being checked, and 1 the right one.
    Operand(usize),
}

/// Builds what a failed check prints: the error rendered now, with operand values filled in while running.
#[must_use]
pub fn failure(checker: &Checker<'_>, diagnostic: Diagnostic, when: &[When]) -> checked::CheckFailure {
    const MARKS: [char; 2] = ['\u{1}', '\u{2}'];
    let mut template = String::new();
    for part in when {
        match part {
            When::Text(text) => template.push_str(text),
            When::Operand(index) => template.push(MARKS[(*index).min(1)]),
        }
    }
    let diagnostic = if when.is_empty() { diagnostic } else { diagnostic.with_when(template) };
    let rendered = diagnostic.render(checker.sources);
    let mut before = String::new();
    let mut when_parts: Vec<(String, usize)> = Vec::new();
    let mut current = String::new();
    let mut first = true;
    for character in rendered.chars() {
        if let Some(index) = MARKS.iter().position(|mark| *mark == character) {
            if first {
                before = std::mem::take(&mut current);
                when_parts.push((String::new(), index));
                first = false;
            } else {
                when_parts.push((std::mem::take(&mut current), index));
            }
        } else {
            current.push(character);
        }
    }
    if first {
        before = std::mem::take(&mut current);
    }
    checked::CheckFailure { before, when: when_parts, after: current }
}

impl Checker<'_> {
    /// Checks a value, given the type it's going into when that's known, since a literal takes its type from where
    /// it lands (D134).
    pub fn expression(
        &mut self,
        body: &mut Body,
        expression: &syntax::Expression,
        expected: Option<&Type>,
    ) -> Check<Typed> {
        match &expression.kind {
            ExpressionKind::Integer(_) => self.integer_literal(expression, expected),
            ExpressionKind::Unary { operator: UnaryOperator::Negate, operand, .. }
                if matches!(operand.kind, ExpressionKind::Integer(_)) =>
            {
                self.integer_literal(expression, expected)
            }
            ExpressionKind::Bool(value) => Ok(Typed::new(Type::Bool, Kind::Bool(*value))),
            ExpressionKind::Text(raw) => {
                let mut errors = Vec::new();
                let text = super::format::plain_text(self.file, raw, expression.span.start as usize, &mut errors);
                let failed = !errors.is_empty();
                self.errors.extend(errors);
                if failed {
                    return Err(Reported);
                }
                Ok(Typed::new(Type::Str, Kind::Text(text.unwrap_or_default())))
            }
            ExpressionKind::Variable(name) => self.read_variable(body, name),
            ExpressionKind::Group(inner) => self.expression(body, inner, expected),
            ExpressionKind::Unary { operator, operand, operator_span } => {
                self.unary(body, expression, *operator, operand, *operator_span, expected)
            }
            ExpressionKind::Binary { operator, operator_span, left, right } => {
                self.binary(body, expression, operator, *operator_span, left, right, expected)
            }
            ExpressionKind::Chain { operands, operators } => self.chain(body, operands, operators),
            ExpressionKind::Call { callee, arguments, .. } => self.call(body, expression, callee, arguments),
            _ => Err(self.not_yet(expression.span, "this kind of value".to_string())),
        }
    }

    /// An integer literal, maybe written with `-` in front.
    fn integer_literal(&mut self, expression: &syntax::Expression, expected: Option<&Type>) -> Check<Typed> {
        let (digits, negative) = match &expression.kind {
            ExpressionKind::Integer(text) => (text.clone(), false),
            ExpressionKind::Unary { operand, .. } => match &operand.kind {
                ExpressionKind::Integer(text) => (text.clone(), true),
                _ => return Err(Reported),
            },
            _ => return Err(Reported),
        };
        let written = self.slice(expression.span).to_string();
        let Ok(magnitude) = digits.replace('_', "").parse::<i128>() else {
            return Err(self.not_yet(expression.span, format!("the number `{written}`")));
        };
        let value = if negative { -magnitude } else { magnitude };
        let kind = match expected {
            Some(Type::Integer { kind, .. }) => *kind,
            _ => IntegerKind::I32,
        };
        let (min, max) = kind.range();
        if value < min || value > max {
            return Err(self.error(Diagnostic::refused(
                expression.span,
                format!("`{written}` doesn't fit `{}`", kind.name()),
                "a literal that can't fit is a compile error (D20)",
            )));
        }
        let mut typed = Typed::new(Type::Integer { kind, bounds: None }, Kind::Integer(value));
        typed.literal = Some(value);
        Ok(typed)
    }

    fn read_variable(&mut self, body: &mut Body, name: &syntax::Name) -> Check<Typed> {
        let Some(index) = body.find(&name.text) else {
            return Err(self.error(Diagnostic::refused(
                name.span,
                format!("`{}` isn't declared", name.text),
                "a variable is declared with `let`, `let mut` or `decl` before it's used (D203)",
            )));
        };
        let variable = body.variables[index].clone();
        match (body.states[index], variable.origin) {
            (State::Assigned, _) => {}
            (_, Origin::RefusedLet) => {
                return Err(self.error(Diagnostic::refused(
                    name.span,
                    format!("`{}` never got a value", name.text),
                    "a variable is read only after it has one (D203)",
                )));
            }
            (state, _) => {
                let place = body.statement.unwrap_or(name.span);
                let what = if state == State::Maybe {
                    format!("`{}` has no value yet on some path here", name.text)
                } else {
                    format!("`{}` has no value yet", name.text)
                };
                return Err(self.error(
                    Diagnostic::refused(place, what, "every path has to `assign` a `decl` before reading it (D203)")
                        .with_because(variable.declared, format!("`{}` is declared here", name.text)),
                ));
            }
        }
        let mut typed = Typed::new(variable.value_type.clone(), Kind::Local(variable.local));
        typed.variable = Some(name.text.clone());
        Ok(typed)
    }

    fn unary(
        &mut self,
        body: &mut Body,
        expression: &syntax::Expression,
        operator: UnaryOperator,
        operand: &syntax::Expression,
        operator_span: Span,
        expected: Option<&Type>,
    ) -> Check<Typed> {
        self.needs_group(operand)?;
        match operator {
            UnaryOperator::Not => {
                let typed = self.condition_value(body, operand)?;
                Ok(Typed::new(Type::Bool, Kind::Not(Box::new(typed.value))))
            }
            UnaryOperator::BitNot => {
                let typed = self.expression(body, operand, expected)?;
                if !typed.value_type.is_integer() {
                    return Err(self.not_yet(expression.span, "`bit_not` on this".to_string()));
                }
                Ok(Typed::new(typed.value_type.plain(), Kind::BitNot(Box::new(typed.value))))
            }
            UnaryOperator::Negate => {
                let typed = self.expression(body, operand, expected)?;
                match &typed.value_type {
                    Type::Integer { kind, .. } => {
                        let failure = failure(
                            self,
                            Diagnostic::not_designed(
                                expression.span,
                                "what happens when unary `-` overflows",
                                NOT_DECIDED_OPERATORS,
                            ),
                            &[
                                When::Text("negating `".into()),
                                When::Operand(0),
                                When::Text(format!("` in `{}`", kind.name())),
                            ],
                        );
                        Ok(Typed::new(
                            typed.value_type.plain(),
                            Kind::Negate { operand: Box::new(typed.value), failure: Some(Box::new(failure)) },
                        ))
                    }
                    Type::Float { .. } => Ok(Typed::new(
                        typed.value_type.plain(),
                        Kind::Negate { operand: Box::new(typed.value), failure: None },
                    )),
                    Type::Unknown => Err(Reported),
                    other => Err(self.error(Diagnostic::refused(
                        operator_span.to(operand.span),
                        format!("maths needs numbers, and this has `{other}`"),
                        CONVERT_WHY,
                    ))),
                }
            }
        }
    }

    /// Refuses a bit operator or an `as` used inside another operator without `( )` (D116).
    fn needs_group(&mut self, operand: &syntax::Expression) -> Check<()> {
        match &operand.kind {
            ExpressionKind::Binary { operator, .. } if operator.is_bits() => Err(self.error(Diagnostic::refused(
                operand.span,
                "this needs `( )` around it, since maths gives bit operators no order",
                "where maths gives no order, you group with `( )` (D116)",
            ))),
            ExpressionKind::As { .. } => Err(self.error(Diagnostic::refused(
                operand.span,
                "this `as` is mixed with other operators, so it needs `( )`",
                "`as` works on a single value, and where maths gives no order you group with `( )` (D116)",
            ))),
            _ => Ok(()),
        }
    }

    /// Whether an expression is a number written straight out, which takes its type from the other side.
    fn is_literal_number(expression: &syntax::Expression) -> bool {
        match &expression.kind {
            ExpressionKind::Integer(_) => true,
            ExpressionKind::Unary { operator: UnaryOperator::Negate | UnaryOperator::BitNot, operand, .. } => {
                Self::is_literal_number(operand)
            }
            ExpressionKind::Group(inner) => Self::is_literal_number(inner),
            ExpressionKind::Binary { operator, left, right, .. } if !operator.is_comparison() => {
                Self::is_literal_number(left) && Self::is_literal_number(right)
            }
            _ => false,
        }
    }

    /// Checks two operands, letting a literal take the type of the other side.
    fn operands(
        &mut self,
        body: &mut Body,
        left: &syntax::Expression,
        right: &syntax::Expression,
        expected: Option<&Type>,
    ) -> Check<(Typed, Typed)> {
        if Self::is_literal_number(left) && !Self::is_literal_number(right) {
            let right_typed = self.expression(body, right, None);
            let hint = right_typed.as_ref().ok().map(|typed| typed.value_type.plain());
            let left_typed = self.expression(body, left, hint.as_ref().or(expected));
            return Ok((left_typed?, right_typed?));
        }
        let both_literal = Self::is_literal_number(left) && Self::is_literal_number(right);
        let left_typed = self.expression(body, left, if both_literal { expected } else { None });
        let hint = left_typed.as_ref().ok().map(|typed| typed.value_type.plain());
        let right_typed = self.expression(body, right, hint.as_ref().or(expected));
        Ok((left_typed?, right_typed?))
    }

    #[allow(clippy::too_many_arguments)]
    fn binary(
        &mut self,
        body: &mut Body,
        expression: &syntax::Expression,
        operator: &BinaryOperator,
        operator_span: Span,
        left: &syntax::Expression,
        right: &syntax::Expression,
        expected: Option<&Type>,
    ) -> Check<Typed> {
        let left_grouped = self.needs_group(left);
        let right_grouped = self.needs_group(right);
        left_grouped?;
        right_grouped?;
        match operator {
            BinaryOperator::And(narrowing) | BinaryOperator::Or(narrowing) => {
                let is_and = matches!(operator, BinaryOperator::And(_));
                let both = match narrowing.as_deref() {
                    None => false,
                    Some("both") => true,
                    Some(other) => {
                        return Err(self.error(Diagnostic::not_designed(
                            operator_span,
                            format!("`{}.{other}`", if is_and { "and" } else { "or" }),
                            NOT_DECIDED,
                        )));
                    }
                };
                let left_typed = self.condition_value(body, left);
                let right_typed = self.condition_value(body, right);
                let (left_value, right_value) = (Box::new(left_typed?.value), Box::new(right_typed?.value));
                let kind = match (is_and, both) {
                    (true, false) => Kind::And(left_value, right_value),
                    (false, false) => Kind::Or(left_value, right_value),
                    (true, true) => Kind::Arithmetic {
                        operator: Arithmetic::BitAnd,
                        left: left_value,
                        right: right_value,
                        failure: None,
                        more: Vec::new(),
                    },
                    (false, true) => Kind::Arithmetic {
                        operator: Arithmetic::BitOr,
                        left: left_value,
                        right: right_value,
                        failure: None,
                        more: Vec::new(),
                    },
                };
                Ok(Typed::new(Type::Bool, kind))
            }
            operator if operator.is_comparison() => {
                let (left_typed, right_typed) = self.operands(body, left, right, None)?;
                let comparison = self.comparison(expression.span, operator, &left_typed, &right_typed)?;
                Ok(Typed::new(
                    Type::Bool,
                    Kind::Compare {
                        operator: comparison,
                        left: Box::new(left_typed.value),
                        right: Box::new(right_typed.value),
                    },
                ))
            }
            BinaryOperator::LastsAtLeast => Err(self.not_yet(expression.span, "`lasts_at_least`".to_string())),
            _ => self.arithmetic(body, expression, operator, operator_span, left, right, expected),
        }
    }

    /// What a comparison means for these two values, or why it's refused.
    fn comparison(&mut self, span: Span, operator: &BinaryOperator, left: &Typed, right: &Typed) -> Check<Comparison> {
        let comparison = match operator {
            BinaryOperator::Equal => Comparison::Equal,
            BinaryOperator::NotEqual => Comparison::NotEqual,
            BinaryOperator::Below => Comparison::Below,
            BinaryOperator::Above => Comparison::Above,
            BinaryOperator::AtLeast => Comparison::AtLeast,
            _ => Comparison::AtMost,
        };
        let (left_type, right_type) = (&left.value_type, &right.value_type);
        if left_type.is_unknown() || right_type.is_unknown() {
            return Err(Reported);
        }
        match (left_type, right_type) {
            (Type::Integer { kind: a, .. }, Type::Integer { kind: b, .. }) if a == b => Ok(comparison),
            (Type::Float { bits: a, kind: x }, Type::Float { bits: b, kind: y }) if a == b && x == y => Ok(comparison),
            (Type::Bool, Type::Bool) => {
                if matches!(comparison, Comparison::Equal | Comparison::NotEqual) {
                    Ok(comparison)
                } else {
                    Err(self.error(Diagnostic::refused(
                        span,
                        format!("`{}` on `bool`", operator_word(operator)),
                        "a `bool` is `true` or `false`, which have no order (D127)",
                    )))
                }
            }
            (Type::Str, Type::Str) => {
                Err(self.error(Diagnostic::not_designed(span, "comparing `str` values", NOT_DECIDED)))
            }
            _ => Err(self.error(Diagnostic::refused(
                span,
                format!("this compares `{left_type}` and `{right_type}`"),
                CONVERT_WHY,
            ))),
        }
    }

    #[allow(clippy::too_many_arguments, clippy::too_many_lines)]
    fn arithmetic(
        &mut self,
        body: &mut Body,
        expression: &syntax::Expression,
        operator: &BinaryOperator,
        operator_span: Span,
        left: &syntax::Expression,
        right: &syntax::Expression,
        expected: Option<&Type>,
    ) -> Check<Typed> {
        let operator_name = self.slice(operator_span).to_string();
        if let BinaryOperator::Divide(None) = operator {
            // A bare `div` is fine on floats (D164), so this waits for the operands' types.
        }
        let (left_typed, right_typed) = self.operands(body, left, right, expected)?;
        let (left_type, right_type) = (left_typed.value_type.clone(), right_typed.value_type.clone());
        if left_type.is_unknown() || right_type.is_unknown() {
            return Err(Reported);
        }
        let numbers = |value_type: &Type| matches!(value_type, Type::Integer { .. } | Type::Float { .. });
        if !numbers(&left_type) || !numbers(&right_type) {
            return Err(self.error(Diagnostic::refused(
                expression.span,
                format!("maths needs numbers, and this has `{left_type}` and `{right_type}`"),
                CONVERT_WHY,
            )));
        }
        if left_type.plain() != right_type.plain() {
            return Err(self.error(Diagnostic::refused(
                expression.span,
                format!("this mixes `{}` and `{}`", left_type.plain(), right_type.plain()),
                CONVERT_WHY,
            )));
        }
        let value_type = left_type.plain();
        let Type::Integer { kind, .. } = value_type else {
            return Err(self.not_yet(expression.span, "maths on floats".to_string()));
        };
        let type_name = kind.name();
        let both = |text: &str| {
            vec![
                When::Text("`".into()),
                When::Operand(0),
                When::Text(format!(" {text} ")),
                When::Operand(1),
                When::Text("`".into()),
            ]
        };
        let pair = || {
            vec![
                When::Text("`".into()),
                When::Operand(0),
                When::Text("` and `".into()),
                When::Operand(1),
                When::Text(format!("` in `{type_name}`")),
            ]
        };
        let divides_by_zero = || {
            Diagnostic::new(
                DiagnosticKind::CheckFailed,
                expression.span,
                "this divides by zero",
                "division by zero traps (D26)",
            )
        };
        let (arithmetic, failure_diagnostic, when) = match operator {
            BinaryOperator::Add => (
                Arithmetic::Add,
                Some(Diagnostic::new(
                    DiagnosticKind::CheckFailed,
                    expression.span,
                    format!("this `+` overflows `{type_name}`"),
                    format!("a plain `+` that overflows traps in `{}` (D42)", self.mode.name()),
                )),
                both("+"),
            ),
            BinaryOperator::Subtract => (
                Arithmetic::Subtract,
                Some(Diagnostic::not_designed(
                    expression.span,
                    "what happens when `-` overflows",
                    NOT_DECIDED_OPERATORS,
                )),
                pair(),
            ),
            BinaryOperator::Multiply => (
                Arithmetic::Multiply,
                Some(Diagnostic::not_designed(
                    expression.span,
                    "what happens when `x` overflows",
                    NOT_DECIDED_OPERATORS,
                )),
                pair(),
            ),
            BinaryOperator::WrappingAdd => (Arithmetic::WrappingAdd, None, Vec::new()),
            BinaryOperator::Divide(narrowing) => match narrowing.as_deref() {
                None => {
                    return Err(self.error(Diagnostic::refused(
                        operator_span,
                        "an integer has no bare `div`, so pick `div.trunc` or `div.floor`",
                        "integer division has no single right answer, so you pick one (D48, D164)",
                    )));
                }
                Some("trunc") => (Arithmetic::DivideTruncate, Some(divides_by_zero()), Vec::new()),
                Some("floor") => (Arithmetic::DivideFloor, Some(divides_by_zero()), Vec::new()),
                Some("exact") => (Arithmetic::DivideExact, Some(divides_by_zero()), Vec::new()),
                Some(_) => {
                    return Err(self.error(Diagnostic::not_designed(
                        operator_span,
                        format!("`{operator_name}`"),
                        NOT_DECIDED_OPERATORS,
                    )));
                }
            },
            BinaryOperator::Remainder(narrowing) => match narrowing.as_deref() {
                None => {
                    return Err(self.error(Diagnostic::refused(
                        operator_span,
                        "there's no bare `mod`, so pick a narrowing",
                        "`mod` has no single right answer, so you pick one (D97)",
                    )));
                }
                Some("trunc") => (Arithmetic::RemainderTruncate, Some(divides_by_zero()), Vec::new()),
                Some("floor") => (Arithmetic::RemainderFloor, Some(divides_by_zero()), Vec::new()),
                Some(_) => {
                    return Err(self.error(Diagnostic::not_designed(
                        operator_span,
                        format!("`{operator_name}`"),
                        NOT_DECIDED_OPERATORS,
                    )));
                }
            },
            BinaryOperator::BitAnd => (Arithmetic::BitAnd, None, Vec::new()),
            BinaryOperator::BitOr => (Arithmetic::BitOr, None, Vec::new()),
            BinaryOperator::BitXor => (Arithmetic::BitXor, None, Vec::new()),
            BinaryOperator::ShiftLeft => (
                Arithmetic::ShiftLeft,
                Some(Diagnostic::not_designed(expression.span, "what happens when `shl` overflows", NOT_DECIDED)),
                pair(),
            ),
            BinaryOperator::ShiftRight => (
                Arithmetic::ShiftRight,
                Some(Diagnostic::not_designed(
                    expression.span,
                    "whether `shr` on a negative number keeps its sign",
                    NOT_DECIDED,
                )),
                vec![When::Text("`".into()), When::Operand(0), When::Text("`".into())],
            ),
            _ => return Err(self.not_yet(operator_span, format!("the operator `{operator_name}`"))),
        };
        let first = failure_diagnostic.map(|diagnostic| Box::new(failure(self, diagnostic, &when)));
        let mut more = Vec::new();
        if matches!(arithmetic, Arithmetic::ShiftLeft | Arithmetic::ShiftRight) {
            // Shifting by the type's width or more isn't decided (standard 12.3).
            more.push(failure(
                self,
                Diagnostic::not_designed(expression.span, "shifting by too much", NOT_DECIDED_OPERATORS),
                &[When::Text("the shift is ".into()), When::Operand(1)],
            ));
        }
        let divides = matches!(
            arithmetic,
            Arithmetic::DivideTruncate
                | Arithmetic::DivideFloor
                | Arithmetic::DivideExact
                | Arithmetic::RemainderTruncate
                | Arithmetic::RemainderFloor
        );
        if arithmetic == Arithmetic::DivideExact {
            more.push(failure(
                self,
                Diagnostic::not_designed(
                    expression.span,
                    "what `div.exact` does when it isn't exact",
                    NOT_DECIDED_OPERATORS,
                ),
                &both("div.exact"),
            ));
        }
        if divides && kind.signed {
            // The lowest value divided by -1 doesn't fit, and what happens isn't decided (standard 12.1).
            more.push(failure(
                self,
                Diagnostic::not_designed(
                    expression.span,
                    format!("the lowest `{type_name}` divided by -1"),
                    "not decided yet (standard 12.1)",
                ),
                &[],
            ));
        }
        Ok(Typed::new(
            value_type,
            Kind::Arithmetic {
                operator: arithmetic,
                left: Box::new(left_typed.value),
                right: Box::new(right_typed.value),
                failure: first,
                more,
            },
        ))
    }

    /// A comparison chain, `1 below $x below 5`: each value once, stopping at the first false (D117, D132).
    fn chain(
        &mut self,
        body: &mut Body,
        operands: &[syntax::Expression],
        operators: &[(BinaryOperator, Span)],
    ) -> Check<Typed> {
        let mut failed = false;
        for operand in operands {
            if self.needs_group(operand).is_err() {
                failed = true;
            }
        }
        if failed {
            return Err(Reported);
        }
        // Values that aren't literals go first, so the literals can take their type.
        let mut typed: Vec<Option<Typed>> = vec![None; operands.len()];
        let mut hint = None;
        for (index, operand) in operands.iter().enumerate() {
            if !Self::is_literal_number(operand) {
                match self.expression(body, operand, None) {
                    Ok(value) => {
                        hint.get_or_insert_with(|| value.value_type.plain());
                        typed[index] = Some(value);
                    }
                    Err(Reported) => failed = true,
                }
            }
        }
        for (index, operand) in operands.iter().enumerate() {
            if typed[index].is_none() && !failed {
                match self.expression(body, operand, hint.as_ref()) {
                    Ok(value) => typed[index] = Some(value),
                    Err(Reported) => failed = true,
                }
            }
        }
        if failed {
            return Err(Reported);
        }
        let typed: Vec<Typed> = typed.into_iter().flatten().collect();
        let mut comparisons = Vec::new();
        for (index, (operator, _)) in operators.iter().enumerate() {
            let span = operands[index].span.to(operands[index + 1].span);
            comparisons.push(self.comparison(span, operator, &typed[index], &typed[index + 1])?);
        }
        // Each operand goes in a temporary, so it's evaluated once, then the comparisons run left to right.
        let locals: Vec<checked::LocalId> =
            typed.iter().map(|each| body.new_local(each.value.machine_type.clone())).collect();
        let read = |index: usize| checked::Expression {
            kind: Kind::Local(locals[index]),
            machine_type: typed[index].value.machine_type.clone(),
        };
        let compare = |index: usize| checked::Expression {
            kind: Kind::Compare {
                operator: comparisons[index],
                left: Box::new(read(index)),
                right: Box::new(read(index + 1)),
            },
            machine_type: MachineType::Bool,
        };
        let bind = |index: usize, inner: checked::Expression| checked::Expression {
            machine_type: inner.machine_type.clone(),
            kind: Kind::Bind {
                local: locals[index],
                value: Box::new(typed[index].value.clone()),
                body: Box::new(inner),
            },
        };
        let last = comparisons.len() - 1;
        let mut result = compare(last);
        for index in (0..last).rev() {
            let later = bind(index + 2, result);
            result = checked::Expression {
                kind: Kind::And(Box::new(compare(index)), Box::new(later)),
                machine_type: MachineType::Bool,
            };
        }
        let result = bind(0, bind(1, result));
        Ok(Typed::new(Type::Bool, result.kind))
    }

    /// A value that has to be a `bool`, as in an `if` or around `and` (D7).
    pub fn condition_value(&mut self, body: &mut Body, expression: &syntax::Expression) -> Check<Typed> {
        let typed = self.expression(body, expression, Some(&Type::Bool))?;
        match &typed.value_type {
            Type::Bool => Ok(typed),
            Type::Unknown => Err(Reported),
            other => Err(self.error(Diagnostic::refused(
                expression.span,
                format!("this is `{other}`, where a `bool` is needed"),
                CONVERT_WHY,
            ))),
        }
    }

    /// A call to a function of the program's own.
    fn call(
        &mut self,
        body: &mut Body,
        expression: &syntax::Expression,
        callee: &syntax::Expression,
        arguments: &[syntax::Argument],
    ) -> Check<Typed> {
        let ExpressionKind::Path(path) = &callee.kind else {
            return Err(self.not_yet(callee.span, "calling this".to_string()));
        };
        let name = self.resolve_path(path);
        let Some(signature) = self.functions.get(&name).cloned() else {
            return Err(self.error(Diagnostic::refused(
                callee.span,
                format!("the function `{name}` isn't declared"),
                "a function is declared with `fn` and `[ ]` (D23)",
            )));
        };
        if arguments.len() != signature.parameters.len() {
            return Err(self.not_yet(expression.span, "a call with a different number of arguments".to_string()));
        }
        let mut values = Vec::new();
        let mut failed = false;
        for (argument, (_, parameter_type, ownership)) in arguments.iter().zip(&signature.parameters) {
            if argument.label.is_some()
                || argument.ownership.is_some()
                || *ownership != syntax::ParameterOwnership::Lent
            {
                failed = true;
                self.not_yet(argument.span, "this argument".to_string());
                continue;
            }
            if !parameter_type.is_copied() {
                failed = true;
                self.not_yet(argument.span, "an argument that isn't a number or `bool`".to_string());
                continue;
            }
            match self.value_into(body, &argument.value, parameter_type) {
                Ok(value) => values.push(checked::Argument::Value(value)),
                Err(Reported) => failed = true,
            }
        }
        if failed {
            return Err(Reported);
        }
        Ok(Typed::new(signature.returns.clone(), Kind::Call { function: signature.id, arguments: values }))
    }

    /// A value going into a place of type `into`: a `let`, a `change`, an argument or a `return`. A literal has to fit
    /// while building, and a value the compiler can't see fits is checked while running (D20).
    pub fn value_into(
        &mut self,
        body: &mut Body,
        expression: &syntax::Expression,
        into: &Type,
    ) -> Check<checked::Expression> {
        let typed = self.expression(body, expression, Some(into))?;
        self.convert_into(expression.span, typed, into)
    }

    /// Checks an already-checked value against where it goes.
    pub fn convert_into(&mut self, span: Span, typed: Typed, into: &Type) -> Check<checked::Expression> {
        if typed.value_type.is_unknown() || into.is_unknown() {
            return Ok(typed.value);
        }
        match (&typed.value_type, into) {
            (Type::Integer { kind: from, bounds: from_bounds }, Type::Integer { kind: to, bounds: to_bounds })
                if from == to =>
            {
                let Some(bounds) = to_bounds else { return Ok(typed.value) };
                if let Some(literal) = typed.literal {
                    if literal < bounds.min || literal > bounds.max {
                        return Err(self.error(Diagnostic::refused(
                            span,
                            format!("`{}` doesn't fit `{into}`", self.slice(span)),
                            "a literal that can't fit is a compile error (D20)",
                        )));
                    }
                    return Ok(typed.value);
                }
                if let Some(from_bounds) = from_bounds
                    && from_bounds.min >= bounds.min
                    && from_bounds.max <= bounds.max
                {
                    return Ok(typed.value);
                }
                Ok(self.fit(span, typed, into, *bounds))
            }
            (from, to) if from == to && !matches!(from, Type::Integer { .. }) => Ok(typed.value),
            (from, to) => Err(self.error(Diagnostic::refused(
                span,
                format!("this is `{from}`, where `{to}` is needed"),
                CONVERT_WHY,
            ))),
        }
    }

    /// Checks while running that a value fits a bound (D20).
    pub fn fit(&mut self, span: Span, typed: Typed, into: &Type, bounds: Bounds) -> checked::Expression {
        let (what, when) = match &typed.variable {
            Some(name) => {
                (format!("`{name}` doesn't fit `{into}`"), vec![When::Text(format!("`{name}` is ")), When::Operand(0)])
            }
            None => (format!("this doesn't fit `{into}`"), vec![When::Text("it is ".into()), When::Operand(0)]),
        };
        let failure = failure(
            self,
            Diagnostic::new(
                DiagnosticKind::CheckFailed,
                span,
                what,
                "a plain `=`, `change` or argument checks a bound when the compiler can't see it fits (D20)",
            ),
            &when,
        );
        let machine_type = typed.value.machine_type.clone();
        checked::Expression {
            kind: Kind::Fit {
                value: Box::new(typed.value),
                min: bounds.min,
                max: bounds.max,
                failure: Box::new(failure),
            },
            machine_type,
        }
    }
}

/// An ordering operator's word, for errors.
fn operator_word(operator: &BinaryOperator) -> &'static str {
    match operator {
        BinaryOperator::Below => "below",
        BinaryOperator::Above => "above",
        BinaryOperator::AtLeast => "at_least",
        BinaryOperator::AtMost => "at_most",
        BinaryOperator::Equal => "==",
        _ => "!=",
    }
}
