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
    /// The value, if this is a float literal, so where it lands can check it fits while building (D158).
    pub float_literal: Option<f64>,
    /// Whether it says `Give` or `Copy`, handing it over (D3).
    pub handed: bool,
}

impl Typed {
    #[must_use]
    pub fn new(value_type: Type, kind: Kind) -> Typed {
        let machine_type = value_type.machine();
        Typed {
            value_type,
            value: checked::Expression { kind, machine_type },
            literal: None,
            variable: None,
            float_literal: None,
            handed: false,
        }
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
    const MARKS: [char; 8] = ['\u{1}', '\u{2}', '\u{3}', '\u{4}', '\u{5}', '\u{6}', '\u{7}', '\u{8}'];
    let mut template = String::new();
    for part in when {
        match part {
            When::Text(text) => template.push_str(text),
            When::Operand(index) => template.push(MARKS[(*index).min(MARKS.len() - 1)]),
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
            ExpressionKind::Float(_) | ExpressionKind::Inexact { .. } | ExpressionKind::Exponent(_) => {
                self.float_literal(expression, expected)
            }
            ExpressionKind::Unary { operator: UnaryOperator::Negate, operand, .. }
                if matches!(operand.kind, ExpressionKind::Float(_)) =>
            {
                self.float_literal(expression, expected)
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
            ExpressionKind::Result => match &body.result_value {
                Some((local, value_type)) => {
                    let value_type = value_type.clone();
                    Ok(Typed::new(value_type, Kind::Local(*local)))
                }
                None => Err(self.not_yet(expression.span, "`result` outside `ensures`".to_string())),
            },
            ExpressionKind::Group(inner) => self.expression(body, inner, expected),
            ExpressionKind::Unary { operator, operand, operator_span } => {
                self.unary(body, expression, *operator, operand, *operator_span, expected)
            }
            ExpressionKind::Binary { operator, operator_span, left, right } => {
                self.binary(body, expression, operator, *operator_span, left, right, expected)
            }
            ExpressionKind::Chain { operands, operators } => self.chain(body, operands, operators),
            ExpressionKind::Path(path) => {
                if let Some(typed) = self.case_value(expression, path)? {
                    return Ok(typed);
                }
                if path.is_single("void") {
                    return Ok(Typed::new(Type::Void, Kind::Integer(0)));
                }
                if path.segments.len() == 1 && !super::clashes(&path.text()) {
                    return Err(self.error(Diagnostic::refused(
                        path.span,
                        format!("`{}` isn't known", path.text()),
                        "variables, parameters and constants start with `$` (D57, D58)",
                    )));
                }
                Err(self.not_yet(expression.span, format!("the value `{}`", path.text())))
            }
            ExpressionKind::Bracket { base, labels, .. } if matches!(base.kind, ExpressionKind::Path(_)) => {
                let ExpressionKind::Path(path) = &base.kind else { return Err(Reported) };
                if let Some(typed) = self.build(body, expression, path, labels)? {
                    return Ok(typed);
                }
                if path.is_single("range") {
                    return Err(self.error(Diagnostic::not_designed(
                        base.span,
                        "a range anywhere but straight after `in`",
                        NOT_DECIDED,
                    )));
                }
                Err(self.not_yet(expression.span, format!("building `{}`", path.text())))
            }
            ExpressionKind::Field { base, field } => self.field(body, base, field),
            ExpressionKind::Owned { ownership, ownership_span, value } => {
                self.owned(body, *ownership, *ownership_span, value, expected)
            }
            ExpressionKind::Bracket { base, .. } if matches!(&base.kind, ExpressionKind::Path(path) if path.is_single("range")) => {
                Err(self.error(Diagnostic::not_designed(
                    base.span,
                    "a range anywhere but straight after `in`",
                    NOT_DECIDED,
                )))
            }
            ExpressionKind::As { value, keyword, written_type } => {
                self.conversion(body, expression, value, keyword, written_type)
            }
            ExpressionKind::Call { callee, arguments, .. } => self.call(body, expression, callee, arguments),
            ExpressionKind::Pass { .. } | ExpressionKind::Instead { .. } | ExpressionKind::Otherwise { .. } => {
                self.handle(body, expression, expected)
            }
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
        if let Some(Type::Float { .. }) = expected {
            return Err(self.error(Diagnostic::not_designed(
                expression.span,
                "an integer literal where a float is needed",
                NOT_DECIDED,
            )));
        }
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

    /// A float literal: `0.5`, `~0.1`, `-1.5`, `exponent ~6.02214076e23` (D59, D158, D165).
    #[allow(clippy::too_many_lines)]
    fn float_literal(&mut self, expression: &syntax::Expression, expected: Option<&Type>) -> Check<Typed> {
        // Peel off what's written around the digits.
        let (mut inner, negative) = match &expression.kind {
            ExpressionKind::Unary { operand, .. } => (operand.as_ref(), true),
            _ => (expression, false),
        };
        let mut marked = false;
        let mut exponent_word = None;
        let mut negative = negative;
        match &inner.kind {
            ExpressionKind::Inexact { literal, exponent } => {
                marked = true;
                exponent_word = *exponent;
                inner = literal.as_ref();
                if let ExpressionKind::Unary { operand, .. } = &inner.kind {
                    negative = true;
                    inner = operand.as_ref();
                }
            }
            ExpressionKind::Exponent(literal) => {
                exponent_word = Some(expression.span);
                inner = literal.as_ref();
                if let ExpressionKind::Unary { operand, .. } = &inner.kind {
                    negative = true;
                    inner = operand.as_ref();
                }
            }
            _ => {}
        }
        let digits_span = inner.span;
        let digits = self.slice(digits_span).to_string();
        let has_exponent = digits.contains(['e', 'E']);
        if has_exponent && digits.contains("e-") {
            return Err(self.error(Diagnostic::not_designed(
                digits_span,
                "a negative exponent",
                "not decided yet (standard 12.2)",
            )));
        }
        if has_exponent && digits.contains("e+") {
            return Err(self.error(Diagnostic::not_designed(
                digits_span,
                "an exponent written with `+`",
                "not decided yet (standard 12.2)",
            )));
        }
        if has_exponent && !digits.split(['e', 'E']).next().unwrap_or("").contains('.') {
            return Err(self.error(Diagnostic::not_designed(
                digits_span,
                "an exponent on an integer",
                "not decided yet (standard 12.2)",
            )));
        }
        if exponent_word.is_some() && !has_exponent {
            return Err(self.error(Diagnostic::refused(
                expression.span,
                "`exponent` goes only on a literal with an exponent",
                "the word says the literal has an `e` in it (D165)",
            )));
        }
        let Some(Type::Float { bits, kind, .. }) = expected else {
            if let Some(Type::Integer { .. }) = expected {
                let value_type = Type::Float { bits: 64, kind: super::types::FloatKind::Stable, bounds: None };
                let mut typed = Typed::new(value_type, Kind::Float(0.0));
                typed.float_literal = Some(0.0);
                return Ok(typed);
            }
            return Err(self.error(Diagnostic::not_designed(
                expression.span,
                "which float type a literal has when nothing around it says",
                NOT_DECIDED,
            )));
        };
        let (bits, kind) = (*bits, *kind);
        let Some(literal) = super::floats::read(&digits, bits, negative) else {
            return Err(self.not_yet(digits_span, format!("the number `{digits}`")));
        };
        let sign = if negative { "-" } else { "" };
        if literal.too_large {
            if kind == super::types::FloatKind::Ieee {
                return Err(self.error(Diagnostic::not_designed(
                    expression.span,
                    format!("a literal too large for `f{bits}.ieee`"),
                    NOT_DECIDED,
                )));
            }
            return Err(self.error(Diagnostic::refused(
                expression.span,
                format!("this literal is too large for `f{bits}`"),
                "a float can't be infinity, and a literal that can't fit is a compile error (D158, D20)",
            )));
        }
        if has_exponent && exponent_word.is_none() {
            let mark = if literal.exact { "" } else { "~" };
            return Err(self.error(Diagnostic::refused_fix(
                expression.span,
                format!("exponent {mark}{sign}{digits}"),
                "a stray `e` can't quietly change a number, so the word goes in front (D165)",
            )));
        }
        let written = if has_exponent { format!("exponent {sign}{digits}") } else { format!("{sign}{digits}") };
        if literal.exact && marked {
            return Err(self.error(Diagnostic::refused_fix(
                expression.span,
                written,
                format!("f{bits} holds {sign}{digits} exactly (D59)"),
            )));
        }
        if !literal.exact && !marked {
            let fixed = if has_exponent { format!("exponent ~{sign}{digits}") } else { format!("~{sign}{digits}") };
            return Err(self.error(Diagnostic::refused_fix(
                expression.span,
                fixed,
                format!("f{bits} can't hold {sign}{digits} exactly (D59)"),
            )));
        }
        let value_type = Type::Float { bits, kind, bounds: None };
        let mut typed = Typed::new(value_type, Kind::Float(literal.value));
        typed.float_literal = Some(literal.value);
        Ok(typed)
    }

    /// `as` builds a new value of another type (D7). What converts to what isn't decided (standard 12.3), past a
    /// number's text (D122, D161) and `.ieee` crossing back (D159).
    fn conversion(
        &mut self,
        body: &mut Body,
        expression: &syntax::Expression,
        value: &syntax::Expression,
        keyword: &syntax::Path,
        written_type: &syntax::Type,
    ) -> Check<Typed> {
        if keyword.segments.len() > 1 {
            return Err(self.not_yet(keyword.span, format!("`{}`", keyword.text())));
        }
        let typed = self.expression(body, value, None)?;
        let into = self.resolve_type(written_type)?;
        let undecided = |from: &Type, into: &Type| {
            Diagnostic::not_designed(
                expression.span,
                format!("converting `{from}` to `{into}` with `as`"),
                "not decided yet (standard 12.3)",
            )
        };
        match (&typed.value_type, &into) {
            (Type::Unknown, _) => Err(Reported),
            (Type::Integer { .. }, Type::Str) => Ok(Typed::new(Type::Str, Kind::TextOf(Box::new(typed.value)))),
            (Type::Float { kind, .. }, Type::Str) if *kind != super::types::FloatKind::Fast => {
                Ok(Typed::new(Type::Str, Kind::TextOf(Box::new(typed.value))))
            }
            (
                Type::Float { bits: from_bits, kind: super::types::FloatKind::Ieee, .. },
                Type::Float { bits, kind, bounds: None },
            ) if from_bits == bits && *kind != super::types::FloatKind::Ieee => {
                let why =
                    format!("`as` out of `f{bits}.ieee` is a real conversion, and it fails on NaN or infinity (D159)");
                let nan = failure(
                    self,
                    Diagnostic::new(
                        DiagnosticKind::CheckFailed,
                        expression.span,
                        format!("this is NaN, so it can't become `{into}`"),
                        why.clone(),
                    ),
                    &[],
                );
                let infinity = failure(
                    self,
                    Diagnostic::new(
                        DiagnosticKind::CheckFailed,
                        expression.span,
                        format!("this is infinity, so it can't become `{into}`"),
                        why,
                    ),
                    &[],
                );
                Ok(Typed::new(
                    into.clone(),
                    Kind::FromIeee { value: Box::new(typed.value), nan: Box::new(nan), infinity: Box::new(infinity) },
                ))
            }
            (from, into) => Err(self.error(undecided(from, into))),
        }
    }

    fn read_variable(&mut self, body: &mut Body, name: &syntax::Name) -> Check<Typed> {
        let Some(index) = body.find(&name.text) else {
            return Err(self.error(Diagnostic::refused(
                name.span,
                format!("`{}` isn't declared", name.text),
                "`let` and `const` declare a variable with its type and value (D57, D83)",
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
            ExpressionKind::Integer(_)
            | ExpressionKind::Float(_)
            | ExpressionKind::Inexact { .. }
            | ExpressionKind::Exponent(_) => true,
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
            (Type::Float { bits: a, kind: x, .. }, Type::Float { bits: b, kind: y, .. }) if a == b && x == y => {
                Ok(comparison)
            }
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
            (Type::Float { bits: a, .. }, Type::Float { bits: b, .. }) if a == b => {
                Err(self.error(Diagnostic::refused(
                    span,
                    format!("this compares `{left_type}` and `{right_type}`"),
                    format!("`f{a}.ieee` is a different type from `f{a}`, and `as` is the wall between them (D159)"),
                )))
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
        if let (Type::Float { bits: a, kind: x, .. }, Type::Float { bits: b, kind: y, .. }) = (&left_type, &right_type)
            && a == b
            && x != y
        {
            use super::types::FloatKind;
            let kinds = [*x, *y];
            if kinds.contains(&FloatKind::Ieee) {
                return Err(self.error(Diagnostic::refused(
                    expression.span,
                    format!("this mixes `{}` and `{}`", left_type.plain(), right_type.plain()),
                    format!("`f{a}.ieee` is a different type from `f{a}`, and `as` is the wall between them (D159)"),
                )));
            }
            return Err(self.error(Diagnostic::not_designed(
                expression.span,
                format!("how `f{a}.fast` and `f{a}.stable` mix"),
                "not decided yet (standard 12.2)",
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
            return self.float_arithmetic(expression, operator, operator_span, value_type, left_typed, right_typed);
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

    /// Float arithmetic: `+`, `-`, `x` and bare `div` (D164). A `.stable` or `.fast` float that becomes infinity or NaN
    /// is a failure (D158), and `.ieee` follows IEEE 754 (D159).
    fn float_arithmetic(
        &mut self,
        expression: &syntax::Expression,
        operator: &BinaryOperator,
        operator_span: Span,
        value_type: Type,
        left: Typed,
        right: Typed,
    ) -> Check<Typed> {
        let Type::Float { bits, kind, .. } = value_type else { return Err(Reported) };
        let arithmetic = match operator {
            BinaryOperator::Add => checked::FloatArithmetic::Add,
            BinaryOperator::Subtract => checked::FloatArithmetic::Subtract,
            BinaryOperator::Multiply => checked::FloatArithmetic::Multiply,
            BinaryOperator::Divide(None) => checked::FloatArithmetic::Divide,
            BinaryOperator::Divide(Some(narrowing)) => {
                return Err(self.error(Diagnostic::not_designed(
                    expression.span,
                    format!("`div.{narrowing}` on floats"),
                    "not decided yet (standard 12.2)",
                )));
            }
            BinaryOperator::Remainder(None) => {
                return Err(self.error(Diagnostic::refused(
                    operator_span,
                    "there's no bare `mod`, so pick a narrowing",
                    "`mod` has no single right answer, so you pick one (D97)",
                )));
            }
            BinaryOperator::Remainder(Some(_)) => {
                return Err(self.error(Diagnostic::not_designed(
                    expression.span,
                    "`mod` on floats",
                    "not decided yet (standard 12.2)",
                )));
            }
            _ => {
                let name = self.slice(operator_span).to_string();
                return Err(self.not_yet(operator_span, format!("`{name}` on floats")));
            }
        };
        let (infinity, nan) = if kind == super::types::FloatKind::Ieee {
            (None, None)
        } else {
            let when = [
                When::Text("`".into()),
                When::Operand(0),
                When::Text("` and `".into()),
                When::Operand(1),
                When::Text(format!("` in `f{bits}`")),
            ];
            let why = "a float can't be NaN or infinity, and making one is a failure (D158)";
            let infinity = failure(
                self,
                Diagnostic::new(DiagnosticKind::CheckFailed, expression.span, "this makes infinity", why),
                &when,
            );
            let nan = failure(
                self,
                Diagnostic::new(DiagnosticKind::CheckFailed, expression.span, "this makes NaN", why),
                &when,
            );
            (Some(Box::new(infinity)), Some(Box::new(nan)))
        };
        Ok(Typed::new(
            value_type,
            Kind::FloatArithmetic {
                operator: arithmetic,
                left: Box::new(left.value),
                right: Box::new(right.value),
                infinity,
                nan,
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
        let typed = self.call_unhandled(body, expression, callee, arguments)?;
        let checked::ExpressionKind::Call { function, .. } = &typed.value.kind else { return Ok(typed) };
        let Some(signature) = self.functions.values().find(|signature| signature.id == *function).cloned() else {
            return Ok(typed);
        };
        if signature.failures.is_empty() {
            return Ok(typed);
        }
        Ok(self.crash_on_failure(expression.span, &signature, typed.value))
    }

    /// A call, before what happens when it fails is known.
    pub fn call_unhandled(
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
        self.old_prints(&name, callee.span)?;
        if name.starts_with("std::") && name.contains('.') && !self.functions.contains_key(&name) {
            // `std` is grouped with `::`, and a `.` after a function is a narrowing (D196).
            let fixed = name.replacen('.', "::", 1);
            return Err(self.error(Diagnostic::refused_fix(
                callee.span,
                fixed,
                "`std` is grouped with `::`, and `.` after a function is a narrowing (D196)",
            )));
        }
        self.require_import(&name, path.span)?;
        let Some(signature) = self.functions.get(&name).cloned() else {
            return Err(self.error(Diagnostic::refused(
                callee.span,
                format!("the function `{name}` isn't declared"),
                "a function is declared with `fn` and `[ ]` (D23)",
            )));
        };
        let matched = self.match_arguments(expression.span, &signature, arguments)?;
        let mut values = Vec::new();
        let mut failed = false;
        let mut changed: Vec<String> = Vec::new();
        for (each, (parameter_name, parameter_type, ownership)) in matched.iter().zip(&signature.parameters) {
            let result = match each {
                super::contracts::Matched::Argument(argument) => {
                    self.argument(body, argument, parameter_name, parameter_type, *ownership, &mut changed)
                }
                super::contracts::Matched::Default(default) => {
                    self.value_into(body, default, parameter_type).map(checked::Argument::Value)
                }
            };
            match result {
                Ok(value) => values.push(value),
                Err(Reported) => failed = true,
            }
        }
        if failed {
            return Err(Reported);
        }
        if super::contracts::needs_site(&signature.top) {
            let site = self.program.sites.len();
            self.program.sites.push(crate::diagnostic::render_because(self.sources, expression.span, "called here"));
            values.push(checked::Argument::Value(checked::Expression {
                kind: Kind::Integer(i128::try_from(site).unwrap_or(0)),
                machine_type: MachineType::Integer { bits: 32, signed: true },
            }));
        }
        let mut typed = Typed::new(signature.returns.clone(), Kind::Call { function: signature.id, arguments: values });
        if let Some(layout) = signature.result {
            typed.value.machine_type = super::failures::result_machine_type(layout);
        }
        Ok(typed)
    }

    /// One argument, with the ownership word the parameter asks for (D2, D3, D184).
    fn argument(
        &mut self,
        body: &mut Body,
        argument: &syntax::Argument,
        parameter_name: &str,
        parameter_type: &Type,
        ownership: syntax::ParameterOwnership,
        changed: &mut Vec<String>,
    ) -> Check<checked::Argument> {
        use syntax::{Ownership, ParameterOwnership};
        let word = argument.ownership.map(|(word, _)| word);
        let shown = self.slice(argument.value.span).to_string();
        let held = is_held(&argument.value);
        match ownership {
            ParameterOwnership::LentMut => {
                let number_why =
                    "a number lent to change says `Lend.mut`, and it's the one word a number takes (D184, D3)";
                match word {
                    Some(Ownership::LendMut) => {}
                    None if !held => {
                        return Err(self.error(Diagnostic::refused(
                            argument.span,
                            format!("`{parameter_name}` changes this, so it says `Lend.mut`"),
                            number_why,
                        )));
                    }
                    _ => {
                        let why = if parameter_type.is_copied() {
                            number_why
                        } else {
                            "every hand-off shows on the page (D3)"
                        };
                        return Err(self.error(Diagnostic::refused_fix(
                            argument.span,
                            format!("Lend.mut {shown}"),
                            why,
                        )));
                    }
                }
                let ExpressionKind::Variable(name) = &argument.value.kind else {
                    return Err(self.not_yet(argument.value.span, "lending this with `Lend.mut`".to_string()));
                };
                if changed.contains(&name.text) {
                    return Err(self.error(Diagnostic::refused(
                        argument.span,
                        format!("`{}` is already handed off with `Lend.mut` in this call", name.text),
                        "`Lend.mut` means nothing else uses it meanwhile, and `Give` means it's gone (D2)",
                    )));
                }
                changed.push(name.text.clone());
                let typed = self.expression(body, &argument.value, Some(parameter_type))?;
                let Some(variable) = body.lookup(&name.text).cloned() else { return Err(Reported) };
                if !variable.mutable {
                    return Err(self.error(Diagnostic::refused(
                        argument.value.span,
                        format!("`{}` can't change, so it can't be lent with `Lend.mut`", name.text),
                        "`let` declares a variable that can't change, and `let mut` one that can (D3)",
                    )));
                }
                if typed.value_type != *parameter_type && !typed.value_type.is_unknown() {
                    return Err(self.error(Diagnostic::refused(
                        argument.value.span,
                        format!("this is `{}`, where `{parameter_type}` is needed", typed.value_type),
                        CONVERT_WHY,
                    )));
                }
                Ok(checked::Argument::Address(variable.local))
            }
            _ if parameter_type.is_copied() => {
                if let Some((_, span)) = argument.ownership {
                    return Err(self.error(Diagnostic::refused_fix(
                        span.to(argument.value.span),
                        shown,
                        "plain numbers and `bool` are always copied, and take no word (D2, D123)",
                    )));
                }
                Ok(checked::Argument::Value(self.value_into(body, &argument.value, parameter_type)?))
            }
            ParameterOwnership::Given => {
                match word {
                    Some(Ownership::Give | Ownership::Copy) | None => {}
                    Some(_) => {
                        return Err(self.error(Diagnostic::refused(
                            argument.span,
                            "this parameter is `Given`, so the call says `Give` or `Copy`",
                            "the call and the parameter both say it, and they must match (D2)",
                        )));
                    }
                }
                if word.is_none() && held {
                    return Err(self.error(Diagnostic::refused(
                        argument.span,
                        "this parameter is `Given`, so the call says `Give` or `Copy`",
                        "the call and the parameter both say it, and they must match (D2)",
                    )));
                }
                let typed = self.expression(body, &argument.value, Some(parameter_type))?;
                Ok(checked::Argument::Value(self.convert_into(argument.value.span, typed, parameter_type)?))
            }
            ParameterOwnership::Lent | ParameterOwnership::LentWritten => {
                match word {
                    Some(Ownership::Lend) => {}
                    None if held => {
                        return Err(self.error(Diagnostic::refused_fix(
                            argument.span,
                            format!("Lend {shown}"),
                            "every hand-off shows on the page (D3)",
                        )));
                    }
                    None => {
                        let value = self.value_into(body, &argument.value, parameter_type)?;
                        return Ok(checked::Argument::Temporary(value));
                    }
                    Some(_) => return Err(self.not_yet(argument.span, "this word on a lent parameter".to_string())),
                }
                let typed = self.expression(body, &argument.value, Some(parameter_type))?;
                let typed_type = typed.value_type.clone();
                if typed_type != *parameter_type && !typed_type.is_unknown() {
                    return Err(self.error(Diagnostic::refused(
                        argument.value.span,
                        format!("this is `{typed_type}`, where `{parameter_type}` is needed"),
                        CONVERT_WHY,
                    )));
                }
                match place_of(body, &argument.value) {
                    Some((local, path)) if path.is_empty() => Ok(checked::Argument::Address(local)),
                    Some((local, path)) => Ok(checked::Argument::FieldAddress { local, path }),
                    None => Ok(checked::Argument::Temporary(typed.value)),
                }
            }
        }
    }

    /// `Give $x` or `Copy $x` where a value is handed over (D2, D3).
    fn owned(
        &mut self,
        body: &mut Body,
        ownership: syntax::Ownership,
        ownership_span: Span,
        value: &syntax::Expression,
        expected: Option<&Type>,
    ) -> Check<Typed> {
        let typed = self.expression(body, value, expected)?;
        if typed.value_type.is_copied() {
            let written = self.slice(value.span).to_string();
            return Err(self.error(Diagnostic::refused_fix(
                ownership_span.to(value.span),
                written,
                "plain numbers and `bool` are always copied, and take no word (D2, D123)",
            )));
        }
        match ownership {
            syntax::Ownership::Give | syntax::Ownership::Copy => {
                let mut typed = typed;
                typed.variable = None;
                typed.handed = true;
                Ok(typed)
            }
            _ => Err(self.not_yet(ownership_span, format!("`{}` here", ownership.word()))),
        }
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
        if !typed.value_type.is_copied() && !typed.handed && is_held(expression) && !typed.value_type.is_unknown() {
            let shown = self.slice(expression.span).to_string();
            return Err(self.error(Diagnostic::refused(
                expression.span,
                format!("`{shown}` is `{}`, so this needs `Give` or `Copy`", typed.value_type),
                "every hand-off shows on the page, including one into a name, a `return` or a field (D3)",
            )));
        }
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
            (
                Type::Float { bits: from_bits, kind: from_kind, bounds: from_bounds },
                Type::Float { bits: to_bits, kind: to_kind, bounds: to_bounds },
            ) if from_bits == to_bits => {
                use super::types::FloatKind;
                if from_kind != to_kind {
                    let from_type = typed.value_type.plain();
                    if *from_kind == FloatKind::Ieee {
                        let written = self.slice(span).to_string();
                        return Err(self.error(Diagnostic::refused_fix(
                            span,
                            format!("{written} as {}", into.plain()),
                            format!("`f{from_bits}.ieee` is a different type from `f{from_bits}`, and `as` is the wall between them (D159)"),
                        )));
                    }
                    if *to_kind == FloatKind::Ieee {
                        return Err(self.error(Diagnostic::refused(
                            span,
                            format!("this is `{from_type}`, where `{into}` is needed"),
                            format!("`f{from_bits}.ieee` is a different type from `f{from_bits}`, and `as` is the wall between them (D159)"),
                        )));
                    }
                    return Err(self.error(Diagnostic::not_designed(
                        span,
                        format!("how `f{from_bits}.fast` and `f{from_bits}.stable` mix"),
                        "not decided yet (standard 12.2)",
                    )));
                }
                let Some(bounds) = to_bounds else { return Ok(typed.value) };
                if let Some(literal) = typed.float_literal {
                    if literal < bounds.min() || literal > bounds.max() {
                        return Err(self.error(Diagnostic::refused(
                            span,
                            format!("`{}` doesn't fit `{into}`", self.slice(span)),
                            "a literal that can't fit is a compile error (D20, D158)",
                        )));
                    }
                    return Ok(typed.value);
                }
                if let Some(from_bounds) = from_bounds
                    && from_bounds.min() >= bounds.min()
                    && from_bounds.max() <= bounds.max()
                {
                    return Ok(typed.value);
                }
                Ok(self.fit_float(span, typed, into, *bounds))
            }
            (from, to) if from == to && !matches!(from, Type::Integer { .. }) => Ok(typed.value),
            (from, to) => Err(self.error(Diagnostic::refused(
                span,
                format!("this is `{from}`, where `{to}` is needed"),
                CONVERT_WHY,
            ))),
        }
    }

    /// Checks while running that a float fits a bound (D20, D158).
    pub fn fit_float(
        &mut self,
        span: Span,
        typed: Typed,
        into: &Type,
        bounds: super::types::FloatBounds,
    ) -> checked::Expression {
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
                "a plain `=`, `change` or argument checks a bound when the compiler can't see it fits (D20, D158)",
            ),
            &when,
        );
        let machine_type = typed.value.machine_type.clone();
        checked::Expression {
            kind: Kind::FitFloat {
                value: Box::new(typed.value),
                min: bounds.min(),
                max: bounds.max(),
                failure: Box::new(failure),
            },
            machine_type,
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

/// Whether a value is held somewhere, a variable or a field of one, rather than fresh, so handing it over needs a
/// word (D3, D182).
fn is_held(expression: &syntax::Expression) -> bool {
    match &expression.kind {
        ExpressionKind::Variable(_) => true,
        ExpressionKind::Field { base, .. } => is_held(base),
        ExpressionKind::Group(inner) => is_held(inner),
        _ => false,
    }
}

/// Where a held value lives: its variable's local, and the fields down to it, by their numbers.
fn place_of(body: &Body, expression: &syntax::Expression) -> Option<(checked::LocalId, Vec<usize>)> {
    match &expression.kind {
        ExpressionKind::Variable(name) => body.lookup(&name.text).map(|variable| (variable.local, Vec::new())),
        ExpressionKind::Group(inner) => place_of(body, inner),
        _ => None,
    }
}
