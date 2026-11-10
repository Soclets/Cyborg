//! Expressions: what each form means, its type, and the checks hidden inside it.

use super::body::root_variable;
use super::hand::HandedOff;
use super::{Checker, Failure, Origin, ParameterInfo, Signature, WHY_TYPES};
use crate::ast::Narrowing;
use crate::ast::{
    Argument, BinaryOperator, CompareOperator, Expression, ExpressionKind, FieldValue,
    FunctionKind, HandWord, Handling, ParameterWord,
};
use crate::ir::{self, Arithmetic, Node, NodeKind, Number};
use crate::report::{Outcome, Report, ReportExtra, Span};
use crate::syntax::is_clashing;
use crate::types::Sources;
use crate::types::{show_float, FloatKind, FloatRules, IntegerKind, Type, Value};

/// A call before anything has said what happens if it fails.
struct RawCall {
    node: Node,
    value_type: Type,
    failures: Vec<Failure>,
    /// A failure with no name yet, and which section of 12 says so.
    unnamed: Option<(&'static str, &'static str)>,
}

impl RawCall {
    fn can_fail(&self) -> bool {
        !self.failures.is_empty() || self.unnamed.is_some()
    }
}

fn expression_digits(expression: &Expression) -> &str {
    match &expression.kind {
        ExpressionKind::Integer { digits, .. } => digits,
        _ => "",
    }
}

/// Whether `as` is only crossing: the same data, with a `type.new` or a unit on or off. (D141)
fn crosses(found: &Type, target: &Type) -> bool {
    match (found, target) {
        (Type::New { underlying, .. }, other) if **underlying == *other => true,
        // A bounded value goes into a plain one with no check, and a `type.new` of one too. (D20)
        (
            Type::New { underlying, .. },
            Type::Integer {
                kind: to_kind,
                bound: None,
                unit: to_unit,
            },
        ) => matches!(
            &**underlying,
            Type::Integer { kind, unit, .. } if kind == to_kind && unit == to_unit
        ),
        (
            Type::Integer {
                kind,
                unit,
                bound: _,
            },
            Type::New { underlying, .. },
        ) => {
            **underlying
                == Type::Integer {
                    kind: *kind,
                    bound: None,
                    unit: unit.clone(),
                }
        }
        // trace: D158
        (
            Type::New { underlying, .. },
            Type::Float {
                kind: to_kind,
                bound: None,
                unit: to_unit,
                rules: to_rules,
            },
        ) => matches!(
            &**underlying,
            Type::Float { kind, unit, rules, .. }
                if kind == to_kind && unit == to_unit && rules == to_rules
        ),
        (
            Type::Float {
                kind, unit, rules, ..
            },
            Type::New { underlying, .. },
        ) => {
            **underlying
                == Type::Float {
                    kind: *kind,
                    bound: None,
                    unit: unit.clone(),
                    rules: *rules,
                }
        }
        (other, Type::New { underlying, .. }) => **underlying == *other,
        (
            Type::Integer {
                kind, unit: from, ..
            },
            Type::Integer {
                kind: to_kind,
                unit: to,
                bound: None,
            },
        ) => kind == to_kind && from.is_some() != to.is_some(),
        (
            Type::Float {
                kind,
                unit: from,
                rules,
                ..
            },
            Type::Float {
                kind: to_kind,
                unit: to,
                rules: to_rules,
                bound: None,
            },
        ) => kind == to_kind && rules == to_rules && from.is_some() != to.is_some(),
        _ => false,
    }
}

/// A number literal's value, with or without a unit behind it.
pub(super) fn literal_number(expression: &Expression) -> Option<i128> {
    match &expression.kind {
        ExpressionKind::WithUnit { number, .. } => literal_integer(number),
        _ => literal_integer(expression),
    }
}

/// The value of an integer literal, written with or without its sign.
pub(super) fn literal_integer(expression: &Expression) -> Option<i128> {
    match &expression.kind {
        ExpressionKind::Integer { digits, negative } => {
            let value: i128 = digits.parse().ok()?;
            Some(if *negative { -value } else { value })
        }
        ExpressionKind::Group(inner) => literal_integer(inner),
        _ => None,
    }
}

pub(super) fn is_number_literal(expression: &Expression) -> bool {
    match &expression.kind {
        ExpressionKind::Integer { .. } | ExpressionKind::Float { .. } => true,
        ExpressionKind::WithUnit { number, .. } | ExpressionKind::Group(number) => {
            is_number_literal(number)
        }
        _ => false,
    }
}

/// Whether `as` is going from `f64.ieee` to its plain `f64`, which is the real conversion. (D159)
fn leaves_ieee(found: &Type, target: &Type) -> bool {
    matches!(
        (found, target),
        (
            Type::Float { kind, unit, rules: FloatRules::Ieee, .. },
            Type::Float { kind: to_kind, unit: to_unit, rules: FloatRules::Plain, bound: None },
        ) if kind == to_kind && unit == to_unit
    )
}

fn node_float(node: &Node) -> Option<f64> {
    match node.kind {
        NodeKind::Constant(Value::F32(value)) => Some(f64::from(value)),
        NodeKind::Constant(Value::F64(value)) => Some(value),
        _ => None,
    }
}

fn number_hint(value_type: Option<&Type>) -> Option<Type> {
    match value_type {
        Some(Type::Integer { kind, .. }) => Some(Type::plain_integer(*kind)),
        Some(Type::Float { kind, rules, .. }) => Some(Type::Float {
            kind: *kind,
            bound: None,
            unit: None,
            rules: *rules,
        }),
        _ => None,
    }
}

/// The number a float literal holds, going through the same checks as any other float literal.
pub(super) fn float_literal_value(expression: &Expression, target: &Type) -> Outcome<f64> {
    let (node, _) = Checker::check_float_literal(expression, Some(target), None)?;
    match node.kind {
        NodeKind::Constant(Value::F32(value)) => Ok(f64::from(value)),
        NodeKind::Constant(Value::F64(value)) => Ok(value),
        _ => Err(Report::refused(
            expression.span,
            "this number can't be read",
            "(D59)",
        )),
    }
}

/// `f64.ieee` and `f64` don't mix, and nothing yet says how `f64.fast` does.
/// trace: D159
pub(super) fn float_rules_clash(
    span: Span,
    what: String,
    found: &Type,
    target: &Type,
) -> Box<Report> {
    let fast = |value: &Type| {
        matches!(
            value,
            Type::Float {
                rules: FloatRules::Fast,
                ..
            }
        )
    };
    if fast(found) || fast(target) {
        return Report::not_designed(span, format!("how `{found}` and `{target}` mix"), "12.2");
    }
    Report::refused(
        span,
        what,
        "`f64.ieee` is a different type from `f64`, and `as` is the wall between them (D159)",
    )
}

/// A value's type, and where it was written.
type TypedSpan = (Type, Span);

/// A type holding borrows, with its sources and what fills them. (D171)
type Filled = (Type, Vec<String>, Vec<String>);

/// A union case with no data, of a union holding borrows, takes its sources from where it lands.
/// (D176)
fn holding_if_expected(found: Type, expected: Option<&Type>) -> Type {
    match expected {
        Some(holding @ Type::Holding { base, .. }) if **base == found => holding.clone(),
        _ => found,
    }
}

/// Whether `text` in decimal is exactly what `value` holds. (D59)
pub(super) fn decimal_is_exact(text: &str, value: f64) -> bool {
    // Formatting with this many places writes a binary float's value out in full.
    let full = format!("{:.1100}", value.abs());
    let normalize = |number: &str| -> (String, String) {
        let (whole, fraction) = number.split_once('.').unwrap_or((number, ""));
        (
            whole.trim_start_matches('0').to_string(),
            fraction.trim_end_matches('0').to_string(),
        )
    };
    normalize(text) == normalize(&full)
}

impl Checker<'_> {
    /// Fits a float into a float of the same width, which is `coerce` for floats.
    /// trace: D158, D159, D20
    fn fit_float(
        &self,
        node: Node,
        found: &Type,
        target: &Type,
        expression: &Expression,
    ) -> Outcome<Node> {
        let (
            Type::Float {
                kind,
                bound: found_bound,
                unit: found_unit,
                rules,
            },
            Type::Float {
                bound: target_bound,
                unit: target_unit,
                rules: target_rules,
                ..
            },
        ) = (found, target)
        else {
            return Ok(node);
        };
        if rules != target_rules {
            let clash = float_rules_clash(
                expression.span,
                format!("this is `{found}`, where `{target}` is needed"),
                found,
                target,
            );
            // Out of `f64.ieee` there's exactly one way to say it, which is the wall itself.
            let fixable = *rules == FloatRules::Ieee
                && *target_rules == FloatRules::Plain
                && target_bound.is_none()
                && found_unit == target_unit;
            return Err(if fixable {
                clash.fix(format!("{} as {target}", self.text_of(expression.span)))
            } else {
                clash
            });
        }
        // A bounded value goes into a plain one with no check. (D20)
        let Some((low, high)) = target_bound else {
            return Ok(node);
        };
        let (low, high) = (low.value(), high.value());
        if let Some(number) = node_float(&node) {
            if number < low || number > high {
                return Err(Report::refused(
                    expression.span,
                    format!("`{}` doesn't fit `{target}`", show_float(*kind, number)),
                    "a literal that can't fit is a compile error (D20, D158)",
                ));
            }
            return Ok(node);
        }
        let inside = found_bound.is_some_and(|(found_low, found_high)| {
            found_low.value() >= low && found_high.value() <= high
        });
        if inside {
            return Ok(node);
        }
        let shown = matches!(
            expression.kind,
            ExpressionKind::Variable(_)
                | ExpressionKind::Field { .. }
                | ExpressionKind::Item { .. }
        )
        .then(|| self.text_of(expression.span).to_string());
        Ok(Node {
            span: expression.span,
            kind: NodeKind::FitFloat {
                value: Box::new(node),
                low,
                high,
                target: target.to_string(),
                shown,
            },
        })
    }

    /// Fits a value into its target, adding the runtime check a bound needs. (D20)
    pub(super) fn coerce(
        &self,
        node: Node,
        found: &Type,
        target: &Type,
        expression: &Expression,
    ) -> Outcome<Node> {
        if found == target {
            return Ok(node);
        }
        self.refuse_unstorable_unit(found, target, expression)?;
        if let (
            Type::Integer {
                kind,
                unit,
                bound: found_bound,
            },
            Type::Integer {
                kind: target_kind,
                unit: target_unit,
                bound: target_bound,
            },
        ) = (found, target)
        {
            if kind == target_kind && self.same_unit(unit.as_ref(), target_unit.as_ref()) {
                // A bounded value goes into a plain `i32` with no check. (D20)
                let Some((low, high)) = target_bound else {
                    return Ok(node);
                };
                if let Some(value) = literal_integer(expression) {
                    // trace: D20
                    if value < *low || value > *high {
                        return Err(Report::refused(
                            expression.span,
                            format!("`{value}` doesn't fit `{target}`"),
                            "a literal that can't fit is a compile error (D20)",
                        ));
                    }
                    return Ok(node);
                }
                if let Some((found_low, found_high)) = found_bound {
                    if found_low >= low && found_high <= high {
                        return Ok(node);
                    }
                }
                let shown = matches!(
                    expression.kind,
                    ExpressionKind::Variable(_)
                        | ExpressionKind::Field { .. }
                        | ExpressionKind::Item { .. }
                )
                .then(|| self.text_of(expression.span).to_string());
                // trace: D20
                return Ok(Node {
                    span: expression.span,
                    kind: NodeKind::Fit {
                        value: Box::new(node),
                        low: *low,
                        high: *high,
                        target: target.to_string(),
                        shown,
                    },
                });
            }
            // trace: D101
            if kind == target_kind && unit.is_none() {
                if let (Some(unit), Some(_)) = (target_unit, literal_integer(expression)) {
                    let shown = self.text_of(expression.span);
                    return Err(Report::refused(
                        expression.span,
                        format!("`{target}` asks for a unit"),
                        "a unit is required where the type asks for one (D101)",
                    )
                    .fix(format!("{shown} {unit}")));
                }
            }
        }
        if let (
            Type::Float {
                kind,
                unit: found_unit,
                rules,
                ..
            },
            Type::Float {
                kind: target_kind,
                unit: target_unit,
                rules: target_rules,
                ..
            },
        ) = (found, target)
        {
            if kind == target_kind
                && (rules != target_rules
                    || self.same_unit(found_unit.as_ref(), target_unit.as_ref()))
            {
                return self.fit_float(node, found, target, expression);
            }
            if kind == target_kind && found_unit != target_unit {
                return Err(Report::refused(
                    expression.span,
                    format!("this is `{found}`, where `{target}` is needed"),
                    "the compiler checks units, and mixing them is refused (D100, D101)",
                ));
            }
        }
        Self::check_held_type(expression.span, target, found)?;
        Ok(node)
    }

    /// trace: D83
    pub(super) fn check_constant_value(
        &mut self,
        value: &Expression,
        target: &Type,
    ) -> Outcome<Node> {
        self.refuse_if_not_constant(value)?;
        self.context.in_constant += 1;
        let result = self.check_into(value, target);
        self.context.in_constant -= 1;
        result
    }

    /// A `const`'s value must be fully known at compile time. (D83, D91)
    pub(super) fn refuse_if_not_constant(&self, value: &Expression) -> Outcome<()> {
        const WHY: &str = "a `const`'s value must be fully known at compile time (D83, D91)";
        match &value.kind {
            ExpressionKind::Integer { .. }
            | ExpressionKind::Float { .. }
            | ExpressionKind::Text(_)
            | ExpressionKind::Empty
            | ExpressionKind::Void
            | ExpressionKind::Path(_) => Ok(()),
            ExpressionKind::WithUnit { number, .. }
            | ExpressionKind::Group(number)
            | ExpressionKind::All(number)
            | ExpressionKind::Not(number)
            | ExpressionKind::Negate(number)
            | ExpressionKind::BitNot(number)
            | ExpressionKind::Bake(number) => self.refuse_if_not_constant(number),
            ExpressionKind::Items(items) => items
                .iter()
                .try_for_each(|item| self.refuse_if_not_constant(item)),
            ExpressionKind::Build { fields, .. } => fields
                .iter()
                .try_for_each(|field| self.refuse_if_not_constant(&field.value)),
            ExpressionKind::Binary { left, right, .. } => {
                self.refuse_if_not_constant(left)?;
                self.refuse_if_not_constant(right)
            }
            ExpressionKind::Compare { first, rest } => {
                self.refuse_if_not_constant(first)?;
                rest.iter()
                    .try_for_each(|(_, item)| self.refuse_if_not_constant(item))
            }
            ExpressionKind::Variable(name) => {
                let is_constant = match self.find_variable(name) {
                    Some(slot) => self
                        .context
                        .variables
                        .get(slot)
                        .is_some_and(|variable| variable.origin == Origin::Constant),
                    None => self.globals.contains_key(name),
                };
                if is_constant {
                    Ok(())
                } else {
                    Err(Report::refused(
                        value.span,
                        format!("`${name}` isn't known at compile time"),
                        WHY,
                    ))
                }
            }
            ExpressionKind::Field { base, .. } => self.refuse_if_not_constant(base),
            ExpressionKind::Item { base, position, .. } => {
                self.refuse_if_not_constant(base)?;
                self.refuse_if_not_constant(position)
            }
            ExpressionKind::Call { path, arguments } => {
                let signature = self.signature_of(&path.text);
                let Some(signature) = signature else {
                    return Err(Report::refused(
                        value.span,
                        format!("`{}` isn't known at compile time", path.text),
                        WHY,
                    ));
                };
                // trace: D91
                if signature.kind != FunctionKind::PureBaked {
                    return Err(Report::refused(
                        value.span,
                        format!("`{}` isn't `fn.pure.baked`", path.text),
                        WHY,
                    ));
                }
                arguments
                    .iter()
                    .try_for_each(|argument| self.refuse_if_not_constant(&argument.value))
            }
            ExpressionKind::Hand { value: inner, .. } | ExpressionKind::As { value: inner, .. } => {
                self.refuse_if_not_constant(inner)
            }
            ExpressionKind::Method {
                receiver,
                arguments,
                ..
            } => {
                self.refuse_if_not_constant(receiver)?;
                arguments
                    .iter()
                    .try_for_each(|argument| self.refuse_if_not_constant(&argument.value))
            }
            _ => Err(Report::refused(
                value.span,
                "this isn't known at compile time",
                WHY,
            )),
        }
    }

    fn signature_of(&self, text: &str) -> Option<&Signature> {
        let text = self.canonical(text);
        let bare = text.strip_prefix("my::").unwrap_or(&text);
        self.signatures.get(bare)
    }

    /// What an expression means when it's read for its value.
    #[allow(clippy::too_many_lines)] // One arm per form of value, read top to bottom.
    pub(super) fn check_expression(
        &mut self,
        expression: &Expression,
        expected: Option<&Type>,
    ) -> Outcome<(Node, Type)> {
        let span = expression.span;
        let simple = |kind: NodeKind| Node { span, kind };
        // A literal takes its type from where it lands, and a `type.new` is one place. (D141)
        // trace: D141
        if let Some(target @ Type::New { underlying, .. }) = expected {
            let is_bool = matches!(
                &expression.kind,
                ExpressionKind::Path(path) if path.text == "true" || path.text == "false"
            );
            if is_bool
                || matches!(
                    expression.kind,
                    ExpressionKind::Integer { .. }
                        | ExpressionKind::Float { .. }
                        | ExpressionKind::WithUnit { .. }
                        | ExpressionKind::Text(_)
                        | ExpressionKind::Empty
                        | ExpressionKind::Items(_)
                        | ExpressionKind::All(_)
                )
            {
                let (node, found) = self.check_expression(expression, Some(underlying))?;
                let node = self.coerce(node, &found, underlying, expression)?;
                return Ok((node, target.clone()));
            }
        }
        match &expression.kind {
            ExpressionKind::Integer { .. } => {
                Self::check_integer_literal(expression, expected, None)
            }
            ExpressionKind::Float { .. } => Self::check_float_literal(expression, expected, None),
            ExpressionKind::WithUnit { number, unit } => {
                if matches!(number.kind, ExpressionKind::Inf) {
                    return Err(Report::refused(
                        span,
                        "`inf` is only for a list's `len.max`",
                        "a list writes `inf` for unbounded (D11)",
                    ));
                }
                // A literal can't say point or span, so it takes either from where it lands.
                // (D249)
                // trace: D249
                let resolved = self.resolve_unit_name(unit)?;
                let resolved = match expected.and_then(|target| match target {
                    Type::Integer { unit, .. } | Type::Float { unit, .. } => unit.as_ref(),
                    _ => None,
                }) {
                    Some(target) if super::units::split_of(target).0 == resolved => target.clone(),
                    _ => resolved,
                };
                if matches!(number.kind, ExpressionKind::Float { .. }) {
                    Self::check_float_literal(number, expected, Some(resolved))
                } else {
                    Self::check_integer_literal(number, expected, Some(resolved))
                }
            }
            ExpressionKind::Text(text) => {
                // Only a format print's text says what `\` means. (D194)
                if let Some(at) = text.find('\\') {
                    let at = span.start + 1 + at;
                    return Err(Report::not_designed(
                        Span::new(at, at + 1),
                        "`\\` in a string",
                        "12.2",
                    ));
                }
                Ok((
                    simple(NodeKind::Constant(Value::Str(text.clone()))),
                    Type::Str,
                ))
            }
            ExpressionKind::Empty => Self::check_empty(span, expected),
            ExpressionKind::Items(items) => self.check_items(items, span, expected),
            ExpressionKind::All(value) => self.check_all(value, span, expected),
            ExpressionKind::Inf => Err(Report::refused(
                span,
                "`inf` is only for a list's `len.max`",
                "a list writes `inf` for unbounded (D11)",
            )),
            ExpressionKind::Void => Err(Report::refused(
                span,
                "`void` isn't a value here",
                "`void` is what a function that returns nothing returns (D11)",
            )),
            ExpressionKind::Variable(_) => {
                let held = self.check_held(expression)?;
                Ok((held.node, held.value_type))
            }
            ExpressionKind::Item {
                base,
                position,
                counts_from_zero,
            } => {
                if root_variable(base).is_some() {
                    let held = self.check_held(expression)?;
                    return Ok((held.node, held.value_type));
                }
                // An item of a fresh value, such as `make()[pos: 1]`, has no variable to walk from.
                let (node, base_type) = self.check_expression(base, None)?;
                let (value_type, position) =
                    self.check_position(base, &base_type, position, *counts_from_zero, span)?;
                Ok((
                    simple(NodeKind::Item(Box::new(node), Box::new(position))),
                    value_type,
                ))
            }
            ExpressionKind::Field { base, name } if root_variable(base).is_none() => {
                // A field of a fresh value, such as `make().n`, has no variable to walk from.
                let (node, base_type) = self.check_expression(base, None)?;
                let (index, value_type) = self.find_field(base, &base_type, name)?;
                Ok((simple(NodeKind::Field(Box::new(node), index)), value_type))
            }
            ExpressionKind::Field { base, name } => {
                let held = self.check_held(base)?;
                if matches!(held.value_type, Type::List { .. } | Type::Array { .. }) {
                    if name.text == "len" {
                        // trace: D2
                        // trace: D140
                        let (Type::List { len_type, .. } | Type::Array { len_type, .. }) =
                            held.value_type
                        else {
                            return Err(Report::refused(span, "this has no length", "(D140)"));
                        };
                        return Ok((
                            simple(NodeKind::Length(Box::new(held.node))),
                            Type::Integer {
                                kind: len_type,
                                bound: None,
                                unit: Some("items".to_string()),
                            },
                        ));
                    }
                    return Err(Report::not_designed(
                        name.span,
                        format!("`.{}` on a collection", name.text),
                        "",
                    ));
                }
                if held.value_type == Type::Str {
                    return Err(Report::not_designed(
                        name.span,
                        format!("`.{}` on a `str`", name.text),
                        "12.2",
                    ));
                }
                let held = self.check_held(expression)?;
                Ok((held.node, held.value_type))
            }
            ExpressionKind::Path(path) => {
                let (node, found) = self.check_path_value(path, span)?;
                Ok((node, holding_if_expected(found, expected)))
            }
            ExpressionKind::Build { path, fields } => {
                self.check_build(path, fields, span, expected)
            }
            ExpressionKind::Call { .. } | ExpressionKind::Method { .. } => {
                // A generic's type parameter can come from where its result goes. (D241)
                self.context.call_expected = expected.cloned();
                let raw = self.check_raw_call(expression);
                self.context.call_expected = None;
                Ok(Self::crash_if_it_fails(raw?, span))
            }
            // trace: D56
            ExpressionKind::Builtin(path)
                if is_clashing(path.text.trim_start_matches('@')) =>
            {
                Err(Report::refused(
                    path.span,
                    format!("`{}` can't happen", path.text),
                    "a builtin looks like a function call and can't be written as a Cyborg function, so `let`, `+` and `as` aren't builtins (D56)",
                ))
            }
            ExpressionKind::Builtin(path) => Err(Report::not_designed(
                path.span,
                format!("the builtin `{}`", path.text),
                "12.9",
            )),
            ExpressionKind::Binary {
                operator,
                left,
                right,
            } => self.check_binary(*operator, left, right, span, expected),
            ExpressionKind::Negate(inner) => self.check_negate(inner, span, expected),
            ExpressionKind::Not(inner) => {
                let node = self.check_condition(inner)?;
                Ok((simple(NodeKind::Not(Box::new(node))), Type::Bool))
            }
            ExpressionKind::BitNot(inner) => {
                let hint = number_hint(expected);
                let (node, found) = self.check_expression(inner, hint.as_ref())?;
                let Type::Integer {
                    kind, unit: None, ..
                } = found
                else {
                    return Err(Report::refused(
                        inner.span,
                        format!("`bit_not` works on a plain integer, and this is `{found}`"),
                        WHY_TYPES,
                    ));
                };
                Ok((
                    simple(NodeKind::BitNot(kind, Box::new(node))),
                    Type::plain_integer(kind),
                ))
            }
            ExpressionKind::Compare { first, rest } => self.check_compare(first, rest, span),
            ExpressionKind::As {
                value,
                target,
                narrowing,
            } => {
                let raw = self.check_as(value, target, narrowing.as_ref(), span)?;
                Ok(Self::crash_if_it_fails(raw, span))
            }
            ExpressionKind::Try(inner) => self.check_try(inner, span),
            ExpressionKind::Fallback { value, fallback } => {
                self.check_fallback(value, fallback, span)
            }
            ExpressionKind::Otherwise { value, handling } => {
                self.check_otherwise(value, handling, span)
            }
            ExpressionKind::Hand { word, .. } => Err(Report::refused(
                span,
                format!("`{}` goes where a value is handed off", word.word()),
                "the words go on a call's argument, or into a name, a `return` or a field (D2, D3)",
            )),
            ExpressionKind::Bake(inner) => self.check_bake(inner, span, expected),
            ExpressionKind::Group(inner) => self.check_expression(inner, expected),
        }
    }

    fn crash_if_it_fails(raw: RawCall, span: Span) -> (Node, Type) {
        if raw.can_fail() {
            // trace: D96
            (
                Node {
                    span,
                    kind: NodeKind::Crash(Box::new(raw.node)),
                },
                raw.value_type,
            )
        } else {
            (raw.node, raw.value_type)
        }
    }

    /// A statement that's a call, which may give back `void` through `?? void`.
    pub(super) fn check_statement_call(&mut self, value: &Expression) -> Outcome<(Node, Type)> {
        self.check_expression(value, None)
    }

    fn check_integer_literal(
        expression: &Expression,
        expected: Option<&Type>,
        unit: Option<String>,
    ) -> Outcome<(Node, Type)> {
        let span = expression.span;
        let Some(value) = literal_integer(expression) else {
            // trace: D20
            return Err(Report::refused(
                span,
                format!(
                    "`{}` doesn't fit any integer here",
                    expression_digits(expression)
                ),
                "a literal that can't fit is a compile error (D20)",
            ));
        };
        if matches!(expected, Some(Type::Float { .. })) {
            return Err(Report::not_designed(
                span,
                "an integer literal where a float is needed",
                "",
            ));
        }
        let kind = match expected {
            Some(Type::Integer { kind, .. }) => *kind,
            _ => IntegerKind::I32,
        };
        // trace: D20
        if !kind.fits(value) {
            return Err(Report::refused(
                span,
                format!("`{value}` doesn't fit `{}`", kind.word()),
                "a literal that can't fit is a compile error (D20)",
            ));
        }
        Ok((
            Node {
                span,
                kind: NodeKind::Constant(Value::Integer(value)),
            },
            Type::Integer {
                kind,
                bound: None,
                unit,
            },
        ))
    }

    /// trace: D59
    pub(super) fn check_float_literal(
        expression: &Expression,
        expected: Option<&Type>,
        unit: Option<String>,
    ) -> Outcome<(Node, Type)> {
        let span = expression.span;
        let ExpressionKind::Float {
            digits,
            negative,
            inexact,
            exponent,
        } = &expression.kind
        else {
            return Err(Report::refused(span, "this number can't be read", "(D59)"));
        };
        let (kind, rules) = match expected {
            Some(Type::Float { kind, rules, .. }) => (*kind, *rules),
            None => {
                return Err(Report::not_designed(
                    span,
                    "which float type a literal has when nothing around it says",
                    "",
                ));
            }
            Some(other) => {
                return Err(Report::refused(
                    span,
                    format!("this is a float, where `{other}` is needed"),
                    WHY_TYPES,
                ));
            }
        };
        let (held, value) = match kind {
            FloatKind::F32 => {
                let parsed: f32 = digits.parse().unwrap_or(f32::INFINITY);
                (
                    f64::from(parsed),
                    Value::F32(if *negative { -parsed } else { parsed }),
                )
            }
            FloatKind::F64 => {
                let parsed: f64 = digits.parse().unwrap_or(f64::INFINITY);
                (parsed, Value::F64(if *negative { -parsed } else { parsed }))
            }
        };
        if held.is_infinite() {
            // trace: D158
            if rules != FloatRules::Ieee {
                return Err(Report::refused(
                    span,
                    format!("this literal is too large for `{}`", kind.word()),
                    "a float can't be infinity, and a literal that can't fit is a compile error (D158, D20)",
                ));
            }
            return Err(Report::not_designed(
                span,
                format!("a literal too large for `{}.ieee`", kind.word()),
                "",
            ));
        }
        let exact = decimal_is_exact(digits, held);
        let sign = if *negative { "-" } else { "" };
        // An exponent literal's fix keeps the way it was written. (D165)
        let (shown, word) = match exponent {
            Some(written) => (format!("{sign}{written}"), "exponent "),
            None => (format!("{sign}{digits}"), ""),
        };
        if exact && *inexact {
            return Err(Report::refused(
                span,
                "this literal is exact, so it takes no `~`",
                format!("{} holds {shown} exactly (D59)", kind.word()),
            )
            .fix(format!("{word}{shown}")));
        }
        if !exact && !*inexact {
            return Err(Report::refused(
                span,
                "this literal gets rounded, so it says so",
                format!("{} can't hold {shown} exactly (D59)", kind.word()),
            )
            .fix(format!("{word}~{shown}")));
        }
        Ok((
            Node {
                span,
                kind: NodeKind::Constant(value),
            },
            Type::Float {
                kind,
                bound: None,
                unit,
                rules,
            },
        ))
    }

    /// trace: D103
    fn check_empty(span: Span, expected: Option<&Type>) -> Outcome<(Node, Type)> {
        let node = |value: Value| Node {
            span,
            kind: NodeKind::Constant(value),
        };
        match expected {
            Some(Type::Str) => Ok((node(Value::Str(String::new())), Type::Str)),
            Some(target @ Type::List { min, .. }) => {
                if min.is_some_and(|min| min > 0) {
                    return Err(Report::refused(
                        span,
                        format!("`{target}` can't be `empty`"),
                        "`empty` is refused where it can't be true, so a list with `len.min` above 0 can't use it (D102, D103)",
                    ));
                }
                Ok((node(Value::List(Vec::new())), target.clone()))
            }
            Some(target @ Type::Map { .. }) => Ok((node(Value::Map), target.clone())),
            Some(other) => Err(Report::refused(
                span,
                format!("`{other}` can't be `empty`"),
                "`empty` is refused where it can't be true (D103)",
            )),
            None => Err(Report::not_designed(
                span,
                "`empty` where nothing says what it's empty of",
                "",
            )),
        }
    }

    /// trace: D102
    fn check_items(
        &mut self,
        items: &[Expression],
        span: Span,
        expected: Option<&Type>,
    ) -> Outcome<(Node, Type)> {
        let count = i128::try_from(items.len()).unwrap_or(i128::MAX);
        let (item_type, target) = match expected {
            Some(
                target @ Type::List {
                    item,
                    min,
                    max,
                    len_type,
                },
            ) => {
                // trace: D140
                if !len_type.fits(count) {
                    return Err(Report::not_designed(
                        span,
                        format!(
                            "a list of `inf` with more items than its `{}` can count",
                            len_type.word()
                        ),
                        "",
                    ));
                }
                if min.is_some_and(|min| count < min) || max.is_some_and(|max| count > max) {
                    return Err(Report::refused(
                        span,
                        format!("`{target}` can't hold {count} items"),
                        "`[ ]` gives a list its items, with the count checked against the type (D102)",
                    ));
                }
                ((**item).clone(), target.clone())
            }
            Some(target @ Type::Array { item, len, .. }) => {
                if count != *len {
                    return Err(Report::refused(
                        span,
                        format!("`{target}` holds exactly {len} items, and this gives {count}"),
                        "`[ ]` gives an array its items, with the count checked against the type (D102)",
                    ));
                }
                ((**item).clone(), target.clone())
            }
            Some(other) => {
                return Err(Report::refused(
                    span,
                    format!("`[ ]` gives a list or array its items, and this is `{other}`"),
                    "`[1, 2, 3]` gives a list or array its items (D102)",
                ));
            }
            None => {
                return Err(Report::not_designed(
                    span,
                    "`[ ]` where nothing says what it's a list of",
                    "",
                ));
            }
        };
        if self.is_pure() && matches!(target, Type::List { .. }) {
            return Err(Report::not_designed(
                span,
                "whether a `fn.pure` may allocate",
                "12.5",
            ));
        }
        let mut nodes = Vec::new();
        for item in items {
            nodes.push(self.check_into(item, &item_type)?);
        }
        Ok((
            Node {
                span,
                kind: NodeKind::Items(nodes),
            },
            target,
        ))
    }

    /// trace: D102
    fn check_all(
        &mut self,
        value: &Expression,
        span: Span,
        expected: Option<&Type>,
    ) -> Outcome<(Node, Type)> {
        let Some(target @ Type::Array { item, len, .. }) = expected else {
            return Err(Report::refused(
                span,
                "`all` fills an array",
                "`= all 0` fills an array with one value (D102)",
            ));
        };
        if !item.is_always_copied() {
            return Err(Report::not_designed(
                span,
                format!("`all` with a `{item}`, and how each item gets its own copy"),
                "",
            ));
        }
        Self::item_limit_check(*len, span)?;
        let node = self.check_into(value, item)?;
        Ok((
            Node {
                span,
                kind: NodeKind::All(Box::new(node), usize::try_from(*len).unwrap_or(usize::MAX)),
            },
            target.clone(),
        ))
    }

    /// `status.ok`, `result` and such.
    fn check_path_value(&mut self, path: &crate::ast::Path, span: Span) -> Outcome<(Node, Type)> {
        let text = self.through_aliases(&self.canonical(&path.text));
        // trace: D127
        if text == "true" || text == "false" {
            return Ok((
                Node {
                    span,
                    kind: NodeKind::Constant(Value::Bool(text == "true")),
                },
                Type::Bool,
            ));
        }
        if text == "result" {
            if self.context.in_ensures {
                let slot = self.context.result_slot;
                return Ok((
                    Node {
                        span,
                        kind: NodeKind::Read(ir::Place {
                            slot,
                            path: Vec::new(),
                        }),
                    },
                    self.context.return_type.clone(),
                ));
            }
            return Err(Report::refused(
                span,
                "`result` is only in `ensures`",
                "`ensures result at_least 0;` says what comes back (D53, D84)",
            ));
        }
        self.refuse_limb_outside_shield(&text, span)?;
        let bare = text.strip_prefix("my::").unwrap_or(&text);
        if let Some((union_name, case_name)) = bare.split_once('.') {
            if let Some(case) = self.find_case(union_name, case_name) {
                let needs_data = self
                    .unions
                    .get(union_name)
                    .and_then(|info| info.cases.get(case))
                    .is_some_and(|(_, fields)| !fields.is_empty());
                if needs_data {
                    return Err(Report::refused(
                        span,
                        format!("`{bare}` has values, which go in `[ ]`"),
                        "a case's values go in `[ ]` (D33)",
                    ));
                }
                return Ok((
                    Node {
                        span,
                        kind: NodeKind::Case(case, Vec::new()),
                    },
                    Type::Union(union_name.to_string()),
                ));
            }
            if self.unions.contains_key(union_name) {
                return Err(Report::refused(
                    span,
                    format!("`{union_name}` has no case `{case_name}`"),
                    "a union's cases are narrowings (D33)",
                ));
            }
        }
        if bare.starts_with("std::") {
            self.require_std(bare.trim_start_matches("std::"), span)?;
            return Err(Report::not_designed(
                span,
                format!("`{bare}` as a value"),
                "",
            ));
        }
        if !bare.contains('.') && !bare.contains(':') && self.structs.contains_key(bare) {
            return Err(Report::refused(
                span,
                format!("building `{bare}` uses `[ ]`"),
                "building one uses `[ ]`, with `=` setting a field (D57)",
            ));
        }
        // trace: D57
        Err(Report::refused(
            span,
            format!("`{text}` isn't known"),
            "variables, parameters and constants start with `$` (D57, D58)",
        ))
    }

    fn check_field_values(
        &mut self,
        declared: &[(String, Type)],
        written: &[FieldValue],
        owner: &str,
        span: Span,
        filled: Option<(&[String], &[String])>,
    ) -> Outcome<Vec<(usize, Node)>> {
        // trace: D57
        if let Some((missing, _)) = declared
            .iter()
            .find(|(name, _)| !written.iter().any(|field| field.name.text == *name))
        {
            return Err(Report::refused(
                span,
                format!("building `{owner}` needs `{missing}`"),
                "building one uses `[ ]`, with `=` setting each field (D57)",
            ));
        }
        let mut nodes = Vec::new();
        for (position, field) in written.iter().enumerate() {
            let Some(index) = declared
                .iter()
                .position(|(name, _)| *name == field.name.text)
            else {
                return Err(Report::refused(
                    field.name.span,
                    format!("`{owner}` has no field `{}`", field.name.text),
                    "building one uses `[ ]`, with `=` setting each field (D57)",
                ));
            };
            if index != position {
                return Err(Report::not_designed(
                    field.name.span,
                    "fields set in a different order from the declaration",
                    "",
                ));
            }
            let mut field_type = declared
                .get(index)
                .map_or(Type::Void, |(_, field_type)| field_type.clone());
            // trace: D171, D176
            if let Some((sources, fills)) = filled {
                field_type = super::hand::fill_sources(&field_type, sources, fills);
                if let Type::Borrow { sources, inner, .. } = &field_type {
                    let node = self.check_borrow_field_value(&field.value, sources, inner)?;
                    nodes.push((index, node));
                    continue;
                }
            }
            nodes.push((index, self.check_into(&field.value, &field_type)?));
        }
        Ok(nodes)
    }

    /// A field that borrows is filled with `Lend` of what its type names. Only reading is
    /// allowed while it lives, so the oracle can hold a copy and see the same thing. (D171)
    /// trace: D171, D176
    fn check_borrow_field_value(
        &mut self,
        value: &Expression,
        sources: &Sources,
        inner: &Type,
    ) -> Outcome<Node> {
        let ExpressionKind::Hand {
            word: HandWord::Lend,
            value: lent,
        } = &value.kind
        else {
            return Err(Report::refused(
                value.span,
                "a field that borrows is filled with `Lend`",
                "a struct names the sources of its borrows, and each is lent from one (D171)",
            ));
        };
        let Sources::Variables(wanted) = sources else {
            let shown = match sources {
                Sources::Named(name) => format!("`{name}`"),
                _ => "program".to_string(),
            };
            return Err(Report::not_designed(
                value.span,
                format!("building a borrow whose source is filled with `{shown}`"),
                "",
            ));
        };
        let held = self.check_lent_place(lent, HandWord::Lend)?;
        let roots = self.borrow_roots(&held.root_name);
        if roots != *wanted {
            let said: Vec<String> = wanted.iter().map(|name| format!("${name}")).collect();
            return Err(Report::refused(
                value.span,
                format!(
                    "this borrows `${}`, and the type says this field's source is `{}`",
                    held.root_name,
                    said.join(", ")
                ),
                "a type fills each source, and every field borrows from what fills its own (D171)",
            ));
        }
        Self::check_held_type(value.span, inner, &held.value_type)?;
        Ok(held.node)
    }

    fn check_build(
        &mut self,
        path: &crate::ast::Path,
        fields: &[FieldValue],
        span: Span,
        expected: Option<&Type>,
    ) -> Outcome<(Node, Type)> {
        let text = self.canonical(&path.text);
        // trace: D134
        if matches!(self.named.get(&text), Some(Type::New { .. })) {
            return Err(Report::not_designed(
                span,
                "building a `type.new` by its own name, rather than crossing with `as`",
                "12.1",
            ));
        }
        let text = self.through_aliases(&text);
        let bare = if let Some(mine) = text.strip_prefix("my::") {
            mine.to_string()
        } else {
            // trace: D75
            if is_clashing(&text) {
                let written = self.text_of(span).to_string();
                return Err(Report::refused(
                    span,
                    format!("`{text}` is built in"),
                    "a name of yours that clashes is written with `my::` (D75)",
                )
                .fix(format!("my::{written}")));
            }
            text.clone()
        };
        let bare = self.built_instance(bare, expected, span)?;
        if let Some(info) = self.struct_info(&bare) {
            let built = Type::Struct(bare.clone());
            let filled = Self::filled_for(&built, &info.sources, expected, span)?;
            let nodes = self.check_field_values(
                &info.fields,
                fields,
                &bare,
                span,
                filled
                    .as_ref()
                    .map(|(_, sources, fills)| (sources.as_slice(), fills.as_slice())),
            )?;
            return Ok((
                Node {
                    span,
                    kind: NodeKind::Struct(nodes),
                },
                filled.map_or(built, |(found, _, _)| found),
            ));
        }
        if let Some((union_name, case_name)) = bare.split_once('.') {
            if let Some(case) = self.find_case(union_name, case_name) {
                let declared = self
                    .unions
                    .get(union_name)
                    .and_then(|info| info.cases.get(case))
                    .map(|(_, declared)| declared.clone())
                    .unwrap_or_default();
                let built = Type::Union(union_name.to_string());
                let (sources, _) = self.sources_of(&built);
                let filled = Self::filled_for(&built, &sources, expected, span)?;
                let nodes = self.check_field_values(
                    &declared,
                    fields,
                    &bare,
                    span,
                    filled
                        .as_ref()
                        .map(|(_, sources, fills)| (sources.as_slice(), fills.as_slice())),
                )?;
                return Ok((
                    Node {
                        span,
                        kind: NodeKind::Case(case, nodes),
                    },
                    filled.map_or(built, |(found, _, _)| found),
                ));
            }
        }
        if bare.starts_with("std::") {
            return Err(Report::not_designed(
                path.span,
                format!("building `{bare}`"),
                "",
            ));
        }
        Err(Report::refused(
            path.span,
            format!("the type `{bare}` isn't known"),
            "building one uses `[ ]`, with `=` setting a field (D57)",
        ))
    }

    fn is_pure(&self) -> bool {
        matches!(
            self.context.kind,
            FunctionKind::Pure | FunctionKind::PureBaked
        )
    }

    /// Checks a call, method or `as` without saying what happens when it fails.
    fn check_raw_call(&mut self, expression: &Expression) -> Outcome<RawCall> {
        let span = expression.span;
        let outermost = self.context.call_span.is_none();
        if outermost {
            self.context.call_span = Some(span);
        }
        let result = match &expression.kind {
            ExpressionKind::Group(inner) => self.check_raw_call(inner),
            ExpressionKind::Call { path, arguments } => self.check_call(path, arguments, span),
            ExpressionKind::Method {
                receiver,
                kit,
                name,
                percent,
                changes,
                arguments,
            } => self.check_method(
                receiver,
                kit.as_deref(),
                name,
                (*percent, *changes),
                arguments,
                span,
            ),
            ExpressionKind::As {
                value,
                target,
                narrowing,
            } => self.check_as(value, target, narrowing.as_ref(), span),
            _ => Err(Report::not_designed(
                span,
                "what `?`, `??` or `otherwise` catches on anything but one call",
                "",
            )),
        };
        if outermost {
            self.context.call_span = None;
        }
        result
    }

    /// Limbs must be imported, and what's dangerous must be in a shield. (D38, D82)
    fn refuse_limb_outside_shield(&self, text: &str, span: Span) -> Outcome<()> {
        let Some(path) = text.strip_prefix('@') else {
            return Ok(());
        };
        // trace: D22
        if path == "std::io::fmt_print" {
            return Err(Report::refused(
                span,
                "`fmt_print` is in `std`'s kit part",
                "`@` goes only on things a limb provides (D22)",
            )
            .fix("std::io::fmt_print"));
        }
        // trace: D225
        if path.starts_with("csup::extern") {
            return Err(Report::not_designed(
                span,
                "how a C function is declared and called through `@csup`",
                "12.5",
            ));
        }
        let dangerous = path.starts_with("csup::") || path == "std::inline.asm";
        if let Some(limb) = path.split("::").next() {
            if limb == "csup" {
                self.require_limb(limb, span)?;
            }
        }
        // trace: D38
        if dangerous && self.context.in_shield == 0 {
            return Err(Report::refused(
                span,
                format!("`{text}` must be inside a shield"),
                "a shield is the only place dangerous code may live (D34, D38)",
            ));
        }
        Err(Report::not_designed(
            span,
            format!("using `{text}` from the oracle, which has no C or assembly in it"),
            "12.9",
        ))
    }

    /// A called name without its `my::`, which only a clashing name writes. (D75)
    fn without_my(text: &str, span: Span) -> Outcome<String> {
        match text.strip_prefix("my::") {
            Some(mine) if !is_clashing(mine) => Err(Report::not_designed(
                span,
                format!("`my::` on `{mine}`, which doesn't clash"),
                "",
            )),
            Some(mine) => Ok(mine.to_string()),
            None => Ok(text.to_string()),
        }
    }

    fn check_call(
        &mut self,
        path: &crate::ast::Path,
        arguments: &[Argument],
        span: Span,
    ) -> Outcome<RawCall> {
        // Only the call itself may hand back a borrow, and not one nested inside it.
        let borrow_wanted = std::mem::take(&mut self.context.borrow_result_wanted);
        let text = self.canonical(&path.text);
        if let Some(report) = self.inline_call(&text, path.span) {
            return Err(report);
        }
        self.refuse_limb_outside_shield(&text, path.span)?;
        if let Some(rest) = text.strip_prefix("std::") {
            self.require_std(rest, path.span)?;
            return self.check_std_call(rest, arguments, span, path.span);
        }
        let bare = Self::without_my(&text, path.span)?;
        let Some(signature) = self.signatures.get(&bare).cloned() else {
            return Err(Report::refused(
                path.span,
                format!("the function `{bare}` isn't declared"),
                "a function is declared with `fn` and `[ ]` (D23)",
            ));
        };
        if signature.kind == FunctionKind::Main {
            return Err(Report::not_designed(path.span, "calling `fn.main`", ""));
        }
        let (signature, tables) = self.fill_if_generic(signature, arguments, span)?;
        self.refuse_impure_call(&signature, path.span)?;
        let parameters = signature.parameters.clone();
        let bound =
            self.bind_arguments(&parameters, signature.any_order, arguments, span, &bare)?;
        let mut handed = Vec::new();
        let mut code = Vec::new();
        let mut written: Vec<&Expression> = Vec::new();
        for (parameter_index, value) in &bound {
            let Some(parameter) = parameters.get(*parameter_index) else {
                continue;
            };
            let argument = self.check_argument(value, parameter, &mut handed)?;
            written.push(value);
            code.push((*parameter_index, argument));
        }
        Self::check_handed_together(&handed, &Self::reads_in_place(&written))?;
        self.check_lasts_at_the_call(&signature, &parameters, &bound, span)?;
        // A struct holding borrows, handed back, borrows what was lent to the call. (D171)
        let mut value_type = signature.return_type.clone();
        if let Type::Holding {
            base,
            sources,
            fills,
        } = &signature.return_type
        {
            let mut filled = Vec::new();
            for fill in fills {
                let name = fill.trim_start_matches('$').to_string();
                let result = self.borrowed_by_call(&[name], &parameters, &bound)?;
                filled.push(match result.roots.as_slice() {
                    [only] => format!("${only}"),
                    _ => fill.clone(),
                });
            }
            value_type = Type::Holding {
                base: base.clone(),
                sources: sources.clone(),
                fills: filled,
            };
        }
        // trace: D170
        if let Type::Borrow {
            sources: Sources::Variables(claimed),
            ..
        } = &signature.return_type
        {
            if !borrow_wanted {
                return Err(Report::not_designed(
                    span,
                    "using a returned borrow anywhere but in a stored borrow or a `return`",
                    "",
                ));
            }
            let result = self.borrowed_by_call(claimed, &parameters, &bound)?;
            self.context.borrowed_result = Some(result);
        }
        Ok(RawCall {
            node: Node {
                span,
                kind: NodeKind::Call {
                    function: signature.index,
                    arguments: code,
                    tables,
                },
            },
            value_type,
            failures: signature.failures.clone(),
            unnamed: None,
        })
    }

    /// Every call meets the function's `requires … lasts_at_least …`. (D171)
    /// trace: D171
    fn check_lasts_at_the_call(
        &self,
        signature: &Signature,
        parameters: &[ParameterInfo],
        bound: &[(usize, &Expression)],
        span: Span,
    ) -> Outcome<()> {
        for (longer, shorter) in &signature.lasts {
            let root = |name: &str| -> Outcome<String> {
                let result = self.borrowed_by_call(&[name.to_string()], parameters, bound)?;
                Ok(result
                    .roots
                    .first()
                    .map_or_else(|| format!("${name}"), |found| format!("${found}")))
            };
            let (long, short) = (root(longer)?, root(shorter)?);
            match self.fill_lasts(&long, &short) {
                Some(true) => {}
                Some(false) => {
                    return Err(Report::refused(
                        span,
                        format!("`{long}` doesn't last as long as `{short}`, and `{}` requires `${longer} lasts_at_least ${shorter}`", signature.name),
                        "every call meets a function's `requires … lasts_at_least …` (D171)",
                    ));
                }
                None => {
                    return Err(Report::refused(
                        span,
                        format!("nothing says `{long}` lasts at least as long as `{short}`, which `{}` requires", signature.name),
                        "a function states which parameter lasts longer with `requires … lasts_at_least …;` (D171)",
                    )
                    .fix_elsewhere(format!("requires {long} lasts_at_least {short};")));
                }
            }
        }
        Ok(())
    }

    /// trace: D86
    fn refuse_impure_call(&self, signature: &Signature, span: Span) -> Outcome<()> {
        let callee_is_pure = matches!(signature.kind, FunctionKind::Pure | FunctionKind::PureBaked);
        if self.context.in_contract && !callee_is_pure {
            return Err(Report::refused(
                span,
                format!(
                    "`{}` isn't pure, so a contract can't call it",
                    signature.name
                ),
                "contracts can only call pure functions (D86)",
            ));
        }
        if self.is_pure() && !callee_is_pure {
            return Err(Report::refused(
                span,
                format!(
                    "`{}` isn't pure, so a pure function can't call it",
                    signature.name
                ),
                "a pure function refuses calling a function that isn't pure (D86)",
            ));
        }
        // trace: D91
        if self.context.kind == FunctionKind::PureBaked && signature.kind != FunctionKind::PureBaked
        {
            return Err(Report::refused(
                span,
                format!(
                    "`{}` isn't `fn.pure.baked`, so a baked function can't call it",
                    signature.name
                ),
                "baked code can call normal functions marked `fn.pure.baked`, and only those (D91)",
            ));
        }
        Ok(())
    }

    /// Matches what's written to the parameters, by label and position, and says what's wrong.
    /// The result is in the order written, with defaults added at the end. (D113, D114)
    /// Building a struct or union that holds borrows takes its sources from the type it's built
    /// into, since nothing else says them. (D171, D176)
    fn filled_for(
        built: &Type,
        sources: &[String],
        expected: Option<&Type>,
        span: Span,
    ) -> Outcome<Option<Filled>> {
        if sources.is_empty() {
            return Ok(None);
        }
        match expected {
            Some(
                holding @ Type::Holding {
                    base,
                    sources,
                    fills,
                },
            ) if **base == *built => Ok(Some((holding.clone(), sources.clone(), fills.clone()))),
            _ => Err(Report::not_designed(
                span,
                format!("building `{built}`, which holds borrows, where nothing says its sources"),
                "",
            )),
        }
    }

    /// At the call, the result's sources become the arguments, and each has to be a named value.
    /// trace: D170, D174
    fn borrowed_by_call(
        &self,
        claimed: &[String],
        parameters: &[ParameterInfo],
        bound: &[(usize, &Expression)],
    ) -> Outcome<super::BorrowedResult> {
        let mut result = super::BorrowedResult::default();
        for name in claimed {
            let Some(index) = parameters
                .iter()
                .position(|parameter| parameter.name == *name)
            else {
                continue;
            };
            let Some((_, argument)) = bound.iter().find(|(found, _)| *found == index) else {
                continue;
            };
            let lent_root = match &argument.kind {
                ExpressionKind::Hand {
                    word: HandWord::Lend | HandWord::LendMut,
                    value,
                } => root_variable(value),
                _ => None,
            };
            let Some(root) = lent_root else {
                return Err(Report::refused(
                    argument.span,
                    "the result borrows from this, so it has to be a named value",
                    "a returned borrow's sources become the arguments, and a fresh one would end with the call (D170)",
                ));
            };
            for original in self.borrow_roots(&root) {
                if !result.roots.contains(&original) {
                    result.roots.push(original);
                }
            }
            if !result.through.contains(&root) {
                result.through.push(root);
            }
        }
        Ok(result)
    }

    fn bind_arguments<'e>(
        &self,
        parameters: &'e [ParameterInfo],
        any_order: bool,
        arguments: &'e [Argument],
        span: Span,
        function_name: &str,
    ) -> Outcome<Vec<(usize, &'e Expression)>> {
        const WHY: &str = "a declared label must be written, and a parameter with no label is called without one (D113, D114)";
        let mut bound: Vec<(usize, &Expression)> = Vec::new();
        let is_bound = |bound: &[(usize, &Expression)], index: usize| {
            bound.iter().any(|(found, _)| *found == index)
        };
        let mut last_labelled: Option<usize> = None;
        for argument in arguments {
            if let Some(label) = &argument.label {
                let Some(index) = parameters
                    .iter()
                    .position(|parameter| parameter.label.as_deref() == Some(label.text.as_str()))
                else {
                    return Err(Report::refused(
                        label.span,
                        format!("`{function_name}` has no label `{}`", label.text),
                        WHY,
                    ));
                };
                if is_bound(&bound, index) {
                    return Err(Report::refused(
                        label.span,
                        format!("`{}` is written twice", label.text),
                        WHY,
                    ));
                }
                if !any_order && last_labelled.is_some_and(|last| last > index) {
                    return Err(Report::refused(
                        label.span,
                        format!("`{}` is out of order", label.text),
                        "labels keep their declared order, unless declared `labels.any_order` (D114)",
                    ));
                }
                last_labelled = Some(index);
                bound.push((index, &argument.value));
            } else {
                let next = parameters
                    .iter()
                    .enumerate()
                    .find(|(index, _)| !is_bound(&bound, *index));
                let Some((index, parameter)) = next else {
                    return Err(Report::refused(
                        argument.value.span,
                        format!("this is one argument too many for `{function_name}`"),
                        WHY,
                    ));
                };
                if let Some(label) = &parameter.label {
                    let shown = self.text_of(argument.value.span);
                    return Err(Report::refused(
                        argument.value.span,
                        format!("`{function_name}` labels this `{label}`"),
                        WHY,
                    )
                    .fix(format!("{label}: {shown}")));
                }
                bound.push((index, &argument.value));
            }
        }
        for (index, parameter) in parameters.iter().enumerate() {
            if is_bound(&bound, index) {
                continue;
            }
            if let Some(default) = &parameter.default {
                // trace: D114
                bound.push((index, default));
                continue;
            }
            let wanted = match &parameter.label {
                Some(label) => format!("`{label}:`"),
                None => format!("`${}`", parameter.name),
            };
            return Err(Report::refused(
                span,
                format!("this call to `{function_name}` leaves off {wanted}"),
                WHY,
            ));
        }
        Ok(bound)
    }

    /// trace: D95, D194
    fn check_std_call(
        &mut self,
        rest: &str,
        arguments: &[Argument],
        span: Span,
        path_span: Span,
    ) -> Outcome<RawCall> {
        // `std` is grouped with `::`, and `.` after a function is a narrowing.
        // trace: D196, D161
        if let Some(inside) = rest.strip_prefix("math.") {
            return Err(Report::refused(
                path_span,
                format!("`std::{rest}` isn't how `std` is laid out"),
                "`std` is grouped with `::`, and `.` after a function is a narrowing (D196)",
            )
            .fix(format!("std::math::{inside}")));
        }
        if rest == "sin" {
            return Err(Report::refused(
                path_span,
                "`sin` is in `std`'s `math` group",
                "`std` is grouped with `::` (D196, D197)",
            )
            .fix("std::math::sin"));
        }
        if let Some(function) = rest
            .strip_prefix("math::")
            .filter(|function| matches!(*function, "sin" | "cos" | "tan"))
        {
            return self.check_trigonometry(function, arguments, span, path_span);
        }
        Self::refuse_old_prints(rest, path_span)?;
        let Some(narrowing) = rest.strip_prefix("io::fmt_print") else {
            return Err(Report::not_designed(
                path_span,
                format!("`std::{rest}`, since `std`'s full contents aren't chosen"),
                "",
            ));
        };
        let (stored, stderr) = match narrowing {
            "" => (false, false),
            ".stored" => (true, false),
            ".stderr" => (false, true),
            // trace: D195, D51
            ".stdout" => {
                return Err(Report::refused(
                    path_span,
                    "`std::io::fmt_print.stdout` spells out what bare `std::io::fmt_print` means",
                    "a default `std` fixes can't be written out, and stdout is the default (D195, D51)",
                )
                .fix("std::io::fmt_print"));
            }
            // trace: D196
            after if after.starts_with("::") => {
                return Err(Report::refused(
                    path_span,
                    "`::` is for paths, and a narrowing after a function is `.`",
                    "`::` reaches into a path, which never runs anything, and a `.` after a function narrows it (D196)",
                )
                .fix(format!("std::io::fmt_print.{}", &after[2..])));
            }
            // They combine, strongest first: `.stored` changes what's printed, and `.stderr` only
            // where it goes. (D221, D223)
            // trace: D221, D223
            ".stored.stderr" => (true, true),
            ".stderr.stored" => {
                let written = self.text_of(path_span);
                return Err(Report::refused(
                    path_span,
                    "narrowings go strongest first, so `.stored` comes before `.stderr`",
                    "the narrowing that changes the action most sits next to the thing, and `.stored` changes what's printed while `.stderr` changes only where (D223)",
                )
                .fix(written.replace(".stderr.stored", ".stored.stderr")));
            }
            other => {
                return Err(Report::not_designed(
                    path_span,
                    format!("`std::io::fmt_print{other}`"),
                    "",
                ));
            }
        };
        self.check_fmt_print(arguments, span, path_span, stored, stderr)
    }

    /// `std::println`, `std::print` and `std::io::fmt_println` aren't the print any more.
    fn refuse_old_prints(rest: &str, path_span: Span) -> Outcome<()> {
        // One print, and every other is gone. (D194, D197)
        // trace: D194, D197
        if rest == "println" || rest.starts_with("println.") {
            return Err(Report::refused(
            path_span,
            format!("`std::{rest}` is gone, and `std` has one print"),
            "`std::io::fmt_print` is the one print, a format print, with `{newln}` for a newline (D194, D197)",
        ));
        }
        if rest == "print" || rest.starts_with("print.") || rest == "io::fmt_println" {
            return Err(Report::refused(
                path_span,
                format!("`std::{rest}` isn't the print"),
                "the print is `std::io::fmt_print` (D197)",
            )
            .fix("std::io::fmt_print"));
        }
        Ok(())
    }

    /// `std::io::fmt_print("{}: {}{newln}", $name, $count)`: a literal format, checked here, and
    /// its values. (D194)
    /// trace: D194, D197
    fn check_fmt_print(
        &mut self,
        arguments: &[Argument],
        span: Span,
        path_span: Span,
        stored: bool,
        stderr: bool,
    ) -> Outcome<RawCall> {
        if self.is_pure() || self.context.in_contract {
            // trace: D86
            return Err(Report::refused(
                path_span,
                "printing is I/O, which a pure function refuses",
                "a pure function refuses I/O, and contracts can only call pure functions (D86)",
            ));
        }
        // trace: D114
        if let Some(label) = arguments
            .iter()
            .find_map(|argument| argument.label.as_ref())
        {
            return Err(Report::refused(
                label.span,
                "a print's text and values have no labels",
                "`std` labels every other argument, except where every one has the same role, like a print's values (D114)",
            ));
        }
        let Some((format, values)) = arguments.split_first() else {
            return Err(Report::not_designed(
                span,
                "`std::io::fmt_print()` with no text",
                "12.1",
            ));
        };
        let ExpressionKind::Text(raw) = &format.value.kind else {
            return Err(Report::not_designed(
                format.value.span,
                "a format that isn't a literal",
                "",
            ));
        };
        let pieces = crate::syntax::parse_format(raw, format.value.span.start + 1)?;
        let holes: Vec<&crate::ast::Hole> = pieces
            .iter()
            .filter_map(|piece| match piece {
                crate::ast::FormatPiece::Hole(hole) => Some(hole),
                crate::ast::FormatPiece::Text(_) => None,
            })
            .collect();
        // A named hole prints its variable, and doesn't mix with the other kinds. (D210, D247)
        // trace: D210, D247
        let named = holes.iter().filter(|hole| hole.name.is_some()).count();
        if named > 0 && named < holes.len() {
            let other = holes.iter().find(|hole| hole.name.is_none());
            return Err(Report::refused(
                other.map_or(format.value.span, |hole| hole.span),
                "this format mixes named holes with `{}` or numbered ones",
                "named holes can't be mixed with `{}` or numbered holes in one format (D247)",
            ));
        }
        let (slots, (value_nodes, value_types)) = if named > 0 {
            if let Some(extra) = values.first() {
                return Err(Report::refused(
                    extra.value.span,
                    "every hole in this format names its variable, so it takes no values",
                    "a literal format is checked when building, including how many values it takes (D194, D210)",
                ));
            }
            ((0..holes.len()).collect(), self.check_named_holes(&holes)?)
        } else {
            (
                Self::hole_slots(&holes, values.len(), format.value.span)?,
                self.check_print_values(values)?,
            )
        };
        let printed = self.print_pieces(pieces, &slots, &value_types, stored)?;
        Ok(RawCall {
            node: Node {
                span,
                kind: NodeKind::FormatPrint {
                    pieces: printed,
                    values: value_nodes,
                    stored,
                    stderr,
                },
            },
            value_type: Type::Void,
            failures: vec![Failure {
                short: "print_failed".to_string(),
                full: "print_failed".to_string(),
                data: Vec::new(),
            }],
            unnamed: None,
        })
    }

    /// The variables named holes print, read as values. (D210)
    fn check_named_holes(
        &mut self,
        holes: &[&crate::ast::Hole],
    ) -> Outcome<(Vec<Node>, Vec<TypedSpan>)> {
        // `{$r.celsius}` reads a field. (D247)
        // trace: D247
        let named_values: Vec<Argument> = holes
            .iter()
            .map(|hole| {
                let name = hole.name.clone().unwrap_or_default();
                let mut pieces = name.split('.');
                let mut value = Expression {
                    span: hole.span,
                    kind: ExpressionKind::Variable(pieces.next().unwrap_or_default().to_string()),
                };
                for field in pieces {
                    value = Expression {
                        span: hole.span,
                        kind: ExpressionKind::Field {
                            base: Box::new(value),
                            name: crate::ast::Name {
                                text: field.to_string(),
                                span: hole.span,
                            },
                        },
                    };
                }
                Argument { label: None, value }
            })
            .collect();
        self.check_print_values(&named_values)
    }

    /// A print reads its values, each in its readable form. (D194)
    fn check_print_values(&mut self, values: &[Argument]) -> Outcome<(Vec<Node>, Vec<TypedSpan>)> {
        let mut value_nodes = Vec::new();
        let mut value_types = Vec::new();
        for argument in values {
            if let ExpressionKind::Hand { .. } = argument.value.kind {
                return Err(Report::not_designed(
                    argument.value.span,
                    "a word on a print's value",
                    "",
                ));
            }
            let (node, found) = self.check_expression(&argument.value, None)?;
            // trace: D139
            if matches!(found, Type::Failure { .. }) {
                return Err(Report::refused(
                    argument.value.span,
                    format!(
                        "`{}` names a failure, which is for reading its data",
                        self.text_of(argument.value.span)
                    ),
                    "`case negative $f:` names the failure, so `$f.val` reads its data (D139)",
                ));
            }
            value_nodes.push(node);
            value_types.push((found, argument.value.span));
        }
        Ok((value_nodes, value_types))
    }

    /// Which value each hole takes, in order or by number, and the counts match. (D194)
    fn hole_slots(holes: &[&crate::ast::Hole], values: usize, span: Span) -> Outcome<Vec<usize>> {
        let numbered = holes.iter().filter(|hole| hole.number.is_some()).count();
        if numbered > 0 && numbered < holes.len() {
            let plain = holes.iter().find(|hole| hole.number.is_none());
            return Err(Report::refused(
                plain.map_or(span, |hole| hole.span),
                "this mixes plain holes and numbered ones",
                "`{1}`, `{2}` are numbered holes, and they can't be mixed with plain ones (D194)",
            ));
        }
        let slots: Vec<usize> = if numbered > 0 {
            holes
                .iter()
                .map(|hole| hole.number.unwrap_or(1).saturating_sub(1))
                .collect()
        } else {
            (0..holes.len()).collect()
        };
        let wanted = slots.iter().copied().max().map_or(0, |most| most + 1);
        if let Some(unused) = (0..values).find(|value| !slots.contains(value)) {
            if wanted == values {
                return Err(Report::refused(
                    span,
                    format!("value {} has no hole", unused + 1),
                    "a literal format is checked when building, and every value goes in a hole (D194)",
                ));
            }
        }
        if wanted != values {
            return Err(Report::refused(
                span,
                format!(
                    "this format takes {wanted} {}, and it's given {values}",
                    if wanted == 1 { "value" } else { "values" }
                ),
                "a literal format is checked when building, including how many values it takes (D194)",
            ));
        }
        Ok(slots)
    }

    /// A shape has to apply to the value it shapes. (D194)
    fn check_shapes(
        found: &Type,
        shapes: &crate::ast::Shapes,
        span: Span,
        stored: bool,
    ) -> Outcome<()> {
        let shown = match found {
            Type::Str => "str",
            Type::Integer { .. } => "integer",
            Type::Float { .. } => "float",
            Type::Bool => {
                return Err(Report::not_designed(span, "how a `bool` prints", "12.3"));
            }
            other => {
                return Err(Report::not_designed(
                    span,
                    format!("how `{other}` prints"),
                    "",
                ));
            }
        };
        let has_unit = matches!(
            found,
            Type::Integer { unit: Some(_), .. } | Type::Float { unit: Some(_), .. }
        );
        let applies = |shape: &str| match shown {
            _ if shape == "unit" => has_unit,
            "str" => matches!(shape, "align:" | "width:"),
            "integer" => !matches!(shape, "places:" | "form:"),
            _ => shape != "base:",
        };
        if let Some((shape, at)) = shapes.written.iter().find(|(shape, _)| !applies(shape)) {
            return Err(Report::refused(
                *at,
                format!("`{shape}` doesn't apply to `{found}`"),
                "a literal format is checked when building, including a shape that doesn't apply (D194)",
            ));
        }
        if shapes.width.is_some() && shapes.align.is_none() && !shapes.zeros {
            return Err(Report::not_designed(
                span,
                "which way a value lines up when only its width is given",
                "",
            ));
        }
        if shapes.zeros && shapes.align.is_some() {
            return Err(Report::not_designed(
                span,
                "`zeros` together with `align:`",
                "",
            ));
        }
        if stored && shown == "float" && (shapes.places.is_some() || shapes.science) {
            return Err(Report::not_designed(
                span,
                "`places:` or `form:` in a `.stored` print",
                "12.2",
            ));
        }
        Ok(())
    }

    /// `.%` lends a receiver read-only, and `.%%` lends it to change, as the method says. (D211)
    /// The span covers the marks as written and the name, so the fix can replace them. The
    /// call wrote the other mark, `%` when it needs `%%` and `%%` when it needs `%`. (D207)
    fn wrong_receiver_mark(name: &crate::ast::Name, changes: bool) -> Box<Report> {
        let (what, mark, written) = if changes {
            (
                "this method changes its receiver, so the call says `.%%`",
                "%%",
                1,
            )
        } else {
            (
                "this method only reads its receiver, so the call says `.%`",
                "%",
                2,
            )
        };
        Report::refused(
            Span::new(name.span.start.saturating_sub(written), name.span.end),
            what,
            "`.%` lends a receiver read-only and `.%%` lends it to change, so the call shows which (D211)",
        )
        .fix(format!("{mark}{}", name.text))
    }

    /// The kit method a short name from `use` stands for, when the receiver is of its type.
    /// trace: D178
    fn kit_method_alias(
        &self,
        kit: Option<&str>,
        percent: bool,
        name: &crate::ast::Name,
        receiver_type: &Type,
    ) -> Option<(String, crate::ast::Name)> {
        if kit.is_some() || !percent {
            return None;
        }
        let (owner, real, _) = self.method_aliases.get(&name.text)?;
        let receiver_owner = match receiver_type {
            Type::List { .. } => "std::list",
            Type::Array { .. } => "std::array",
            Type::Map { ordered: true, .. } => "std::map.tree",
            Type::Map { ordered: false, .. } => "std::map.hash",
            _ => return None,
        };
        if owner != receiver_owner {
            return None;
        }
        let (kit, _) = owner.split_once("::")?;
        Some((
            kit.to_string(),
            crate::ast::Name {
                text: real.clone(),
                span: name.span,
            },
        ))
    }

    /// `$xs.std::%append(val: 4)` is the one method that's decided. (D4, D115)
    #[allow(clippy::too_many_arguments)] // A method call has this many parts.
    #[allow(clippy::too_many_lines)] // Every way to spell a method wrong, top to bottom.
    fn check_method(
        &mut self,
        receiver: &Expression,
        kit: Option<&str>,
        name: &crate::ast::Name,
        (percent, changes): (bool, bool),
        arguments: &[Argument],
        span: Span,
    ) -> Outcome<RawCall> {
        let receiver_text = self.text_of(receiver.span).to_string();
        let call_text = self.text_of(span);
        let after_name = call_text
            .split_once(&name.text)
            .map_or("", |(_, rest)| rest)
            .to_string();
        let correct = format!("{receiver_text}.std::%%{}{after_name}", name.text);
        // A short name from `use` keeps its method's mark. (D211)
        // trace: D211
        if let (None, Some((_, _, alias_changes))) = (kit, self.method_aliases.get(&name.text)) {
            if percent && changes != *alias_changes {
                return Err(Self::wrong_receiver_mark(name, *alias_changes));
            }
        }
        if let Some(limb) = kit.filter(|kit| kit.starts_with('@')) {
            let path = format!("{limb}::{}", name.text);
            self.refuse_limb_outside_shield(&path, name.span)?;
        }
        // `reading.%warm_up(…)` names the method in full, and D130 only shows `$r.%warm_up(…)`.
        if let ExpressionKind::Path(path) = &receiver.kind {
            if self.resolve_user_type(&path.text).is_some() {
                return Err(Report::not_designed(
                    span,
                    "calling a method by its full name, as a plain function",
                    "",
                ));
            }
        }
        let (fresh, receiver_type) = if root_variable(receiver).is_some() {
            (None, self.check_held(receiver)?.value_type)
        } else {
            let (node, found) = self.check_expression(receiver, None)?;
            (Some(node), found)
        };
        // A short name from `use` stands for the kit method it names, on that type only.
        // trace: D178
        let resolved = self.kit_method_alias(kit, percent, name, &receiver_type);
        let (kit, name) = match &resolved {
            Some((kit, real)) => (Some(kit.as_str()), real),
            None => (kit, name),
        };
        // A method of your own is declared with `%` or `%%`, and found under either. (D211)
        let own_method = match &receiver_type {
            Type::Struct(type_name)
            | Type::Union(type_name)
            | Type::New {
                name: type_name, ..
            } => {
                let changing = format!("{type_name}.%%{}", name.text);
                if self.signatures.contains_key(&changing) {
                    Some(changing)
                } else {
                    Some(format!("{type_name}.%{}", name.text))
                }
            }
            _ => None,
        };
        // trace: D8
        if !percent {
            let report = Report::refused(
                name.span,
                "a `.` never runs code, and a `.%` always does",
                "a `.` never runs code, and a `.%` always does (D8)",
            );
            let is_own = own_method
                .as_ref()
                .is_some_and(|method| kit.is_none() && self.signatures.contains_key(method));
            let own_mark = if own_method
                .as_ref()
                .is_some_and(|method| method.contains(".%%"))
            {
                "%%"
            } else {
                "%"
            };
            return Err(if is_own {
                let mut fixed = report.fix(format!(
                    "{receiver_text}.{own_mark}{}{after_name}",
                    name.text
                ));
                fixed.span = span;
                fixed
            } else if name.text == "append" {
                let mut fixed = report.fix(correct);
                fixed.span = span;
                fixed
            } else {
                report
            });
        }
        let is_std_list_method = matches!(receiver_type, Type::List { .. });
        if let (None, Some(method)) = (kit, &own_method) {
            let Some(signature) = self.signatures.get(method).cloned() else {
                return Err(Report::refused(
                    name.span,
                    format!("`{receiver_type}` has no method `{}`", name.text),
                    "a method is declared with its type in the name (D130)",
                ));
            };
            // trace: D211
            let declared_changes = method.contains(".%%");
            if changes != declared_changes {
                return Err(Self::wrong_receiver_mark(name, declared_changes));
            }
            return self.check_own_method(receiver, fresh, &signature, arguments, span);
        }
        match kit {
            None => {
                // trace: D115
                if is_std_list_method && name.text == "append" {
                    return Err(Report::refused(
                        span,
                        "`append` is from `std`, and a method from a kit says its path",
                        "a method from a kit says its path, because bare means base or your own (D115)",
                    )
                    .fix(correct));
                }
                return Err(Report::not_designed(
                    span,
                    format!("a method of your own on `{receiver_type}`"),
                    "",
                ));
            }
            Some("std") => {}
            Some(other) => {
                return Err(Report::not_designed(
                    span,
                    format!("a method from `{other}`"),
                    "12.9",
                ));
            }
        }
        self.require_std(&format!("list.%{}", name.text), span)?;
        if name.text != "append" || !is_std_list_method {
            return Err(Report::not_designed(
                name.span,
                format!(
                    "the method `std::%{}`, since `std`'s full contents aren't chosen",
                    name.text
                ),
                "",
            ));
        }
        // trace: D211
        if !changes {
            return Err(Self::wrong_receiver_mark(name, true));
        }
        if self.is_pure() {
            return Err(Report::not_designed(
                span,
                "whether a `fn.pure` may allocate",
                "12.5",
            ));
        }
        // trace: D5
        let held = self.check_lent_place(receiver, crate::ast::HandWord::LendMut)?;
        let (
            Some(place),
            Type::List {
                item,
                max,
                len_type,
                ..
            },
        ) = (held.place.clone(), held.value_type.clone())
        else {
            return Err(Report::refused(
                receiver.span,
                "this can't change",
                "`Lend.mut` needs something that can change (D3)",
            ));
        };
        let parameters = vec![ParameterInfo {
            name: "item".to_string(),
            label: Some("val".to_string()),
            word: ParameterWord::Given,
            parameter_type: (*item).clone(),
            default: None,
            label_has_unit: false,
        }];
        let bound = self.bind_arguments(&parameters, false, arguments, span, "append")?;
        let mut handed = vec![HandedOff {
            root: held.root_name.clone(),
            path: place.keys(),
            word: crate::ast::HandWord::LendMut,
            span: receiver.span,
        }];
        let mut item_node = None;
        let mut written: Vec<&Expression> = Vec::new();
        for (parameter_index, value) in &bound {
            let Some(parameter) = parameters.get(*parameter_index) else {
                continue;
            };
            written.push(value);
            match self.check_argument(value, parameter, &mut handed)? {
                ir::Argument::Value(node) => item_node = Some(node),
                ir::Argument::Alias(aliased) => {
                    item_node = Some(Node {
                        span: value.span,
                        kind: NodeKind::Read(aliased),
                    });
                }
            }
        }
        Self::check_handed_together(&handed, &Self::reads_in_place(&written))?;
        let Some(item_node) = item_node else {
            return Err(Report::refused(
                span,
                "this call to `append` leaves off `val:`",
                "a declared label must be written (D113)",
            ));
        };
        Ok(RawCall {
            node: Node {
                span,
                kind: NodeKind::Append {
                    receiver: place,
                    shown: receiver_text,
                    item: Box::new(item_node),
                    max,
                    count_limit: len_type.range().1,
                },
            },
            value_type: Type::Void,
            failures: Vec::new(),
            unnamed: Some(("what the failure is called when a list is full", "12.7")),
        })
    }

    /// `$r.%warm_up(by: 2)`: the signature's first parameter says what `%` leaves unsaid.
    /// trace: D4, D5, D130
    fn check_own_method(
        &mut self,
        receiver: &Expression,
        fresh: Option<Node>,
        signature: &Signature,
        arguments: &[Argument],
        span: Span,
    ) -> Outcome<RawCall> {
        self.refuse_impure_call(signature, span)?;
        // trace: D91
        if self.context.in_constant > 0 && signature.kind != FunctionKind::PureBaked {
            return Err(Report::refused(
                span,
                format!("`{}` isn't `fn.pure.baked`", signature.name),
                "a `const`'s value must be fully known at compile time (D83, D91)",
            ));
        }
        let Some((held_as, rest)) = signature.parameters.split_first() else {
            return Err(Report::refused(
                span,
                "this method has no receiver",
                "(D130)",
            ));
        };
        let mut handed = Vec::new();
        let mut code = Vec::new();
        let receiver_argument = match (fresh, held_as.word) {
            (Some(node), ParameterWord::Bare) => ir::Argument::Value(node),
            (Some(_), _) => {
                return Err(Report::not_designed(
                    receiver.span,
                    "lending a fresh value with `Lend.mut`",
                    "",
                ));
            }
            (None, word) => {
                let hand = if word == ParameterWord::LentMut {
                    HandWord::LendMut
                } else {
                    HandWord::Lend
                };
                let held = self.check_lent_place(receiver, hand)?;
                handed.push(HandedOff {
                    root: held.root_name.clone(),
                    path: held.place.as_ref().map(ir::Place::keys).unwrap_or_default(),
                    word: hand,
                    span: receiver.span,
                });
                match held.place {
                    Some(place) => ir::Argument::Alias(place),
                    None => ir::Argument::Value(held.node),
                }
            }
        };
        code.push((0, receiver_argument));
        let bound =
            self.bind_arguments(rest, signature.any_order, arguments, span, &signature.name)?;
        let mut written: Vec<&Expression> = Vec::new();
        for (parameter_index, value) in &bound {
            let Some(parameter) = rest.get(*parameter_index) else {
                continue;
            };
            let argument = self.check_argument(value, parameter, &mut handed)?;
            written.push(value);
            code.push((*parameter_index + 1, argument));
        }
        Self::check_handed_together(&handed, &Self::reads_in_place(&written))?;
        Ok(RawCall {
            node: Node {
                span,
                kind: NodeKind::Call {
                    function: signature.index,
                    arguments: code,
                    tables: Vec::new(),
                },
            },
            value_type: signature.return_type.clone(),
            failures: signature.failures.clone(),
            unnamed: None,
        })
    }

    /// `Give $r as wrapped` and `Copy $r as wrapped`. (D149)
    /// trace: D149
    fn check_word_on_as(
        &mut self,
        value: &Expression,
        word: HandWord,
        inner: &Expression,
    ) -> Outcome<(Node, Type)> {
        // The word is on a plain number, or on a fresh value, so nothing's left behind. (D123, D2)
        if root_variable(inner).is_none() {
            return Err(Report::refused(
                value.span,
                "a fresh value takes no word",
                "a fresh value that has no holder yet takes no word (D2, D149)",
            )
            .fix(self.text_of(inner.span).to_string()));
        }
        let held_type = self.check_held(inner)?.value_type;
        if held_type.is_always_copied() {
            return Err(Self::word_on_a_copied_value(
                value,
                self.text_of(inner.span),
                &held_type,
            ));
        }
        let held = match word {
            HandWord::Give => self.check_give(inner, value.span)?,
            HandWord::Copy => self.check_copy(inner)?,
            HandWord::Lend | HandWord::LendMut => {
                return Err(Report::refused(
                    value.span,
                    format!(
                        "`as` builds a new value, so `{}` doesn't fit here",
                        word.word()
                    ),
                    "`as` on a whole value says `Give` or `Copy` (D3, D149)",
                ));
            }
        };
        Ok((held.node, held.value_type))
    }

    /// trace: D7
    fn check_as(
        &mut self,
        value: &Expression,
        target: &crate::ast::TypeExpression,
        narrowing: Option<&crate::ast::Name>,
        span: Span,
    ) -> Outcome<RawCall> {
        // trace: D116
        if !matches!(
            value.kind,
            ExpressionKind::Variable(_)
                | ExpressionKind::Field { .. }
                | ExpressionKind::Item { .. }
                | ExpressionKind::Integer { .. }
                | ExpressionKind::Float { .. }
                | ExpressionKind::Text(_)
                | ExpressionKind::Path(_)
                | ExpressionKind::Build { .. }
                | ExpressionKind::Empty
                | ExpressionKind::Bake(_)
                | ExpressionKind::Group(_)
                | ExpressionKind::Call { .. }
                | ExpressionKind::Method { .. }
                | ExpressionKind::Try(_)
                | ExpressionKind::WithUnit { .. }
                | ExpressionKind::As { .. }
                | ExpressionKind::Hand { .. }
        ) {
            return Err(Report::refused(
                value.span,
                "this `as` is on more than a single value, so it needs `( )`",
                "`as` works on a single value, and where maths gives no order you group with `( )` (D116)",
            ));
        }
        let target_type = self.resolve_type(target)?;
        // A literal takes its type from where it lands, even here. (D141)
        // `as str` says nothing about which float a literal is, so that stays unsaid. (D161)
        let float_to_text =
            target_type == Type::Str && matches!(value.kind, ExpressionKind::Float { .. });
        let (node, found) = if is_number_literal(value) && !float_to_text {
            self.check_expression(value, Some(&target_type))?
        } else if let ExpressionKind::Hand { word, value: inner } = &value.kind {
            self.check_word_on_as(value, *word, inner)?
        } else {
            self.check_expression(value, None)?
        };
        // trace: D141
        if crosses(&found, &target_type) {
            // trace: D149
            if !matches!(value.kind, ExpressionKind::Hand { .. })
                && !found.is_always_copied()
                && root_variable(value).is_some()
            {
                return Err(Report::refused(
                    value.span,
                    format!(
                        "`{}` is `{found}`, so this says `Give` or `Copy`",
                        self.text_of(value.span)
                    ),
                    "`as` on a whole value says its ownership: `Give` leaves it dead, and `Copy` pays for a copy (D3, D149)",
                ));
            }
            return Ok(RawCall {
                node,
                value_type: target_type,
                failures: Vec::new(),
                unnamed: None,
            });
        }
        self.check_real_as(node, &found, target_type, narrowing, span)
    }

    /// `as` when it isn't only crossing, so it builds a new value and can fail.
    fn check_real_as(
        &self,
        node: Node,
        found: &Type,
        target_type: Type,
        narrowing: Option<&crate::ast::Name>,
        span: Span,
    ) -> Outcome<RawCall> {
        // trace: D83
        if self.context.in_constant > 0 {
            return Err(Report::refused(
                span,
                "this `as` isn't known at compile time",
                "a `const`'s value must be fully known at compile time (D83, D91)",
            ));
        }
        // trace: D159
        if leaves_ieee(found, &target_type) {
            return Ok(RawCall {
                node: Node {
                    span,
                    kind: NodeKind::FloatFromIeee {
                        value: Box::new(node),
                        target: target_type.to_string(),
                    },
                },
                value_type: target_type,
                failures: Vec::new(),
                unnamed: Some(("what a failed `as` is called", "12.3")),
            });
        }
        if self.is_pure() {
            return Err(Report::not_designed(
                span,
                "whether a `fn.pure` may allocate, as `as` can",
                "12.5",
            ));
        }
        // trace: D122
        // trace: D161
        if let (Type::Str, Type::Float { kind, rules, .. }) = (&target_type, found) {
            return Ok(RawCall {
                node: Node {
                    span,
                    kind: NodeKind::FloatToStr(Number::Float(*kind, *rules), Box::new(node)),
                },
                value_type: Type::Str,
                failures: Vec::new(),
                unnamed: Some(("what a failed `as` is called", "12.3")),
            });
        }
        // trace: D255
        if self.is_unit_conversion(found, &target_type) {
            let (node, value_type) =
                self.convert_unit(node, found, &target_type, narrowing, span)?;
            return Ok(RawCall {
                node,
                value_type,
                failures: Vec::new(),
                unnamed: None,
            });
        }
        if let Some(narrowing) = narrowing {
            return Err(Report::not_designed(
                narrowing.span,
                format!("`as.{}` on anything but a change of unit", narrowing.text),
                "12.2",
            ));
        }
        let converts = target_type == Type::Str && matches!(found, Type::Integer { .. });
        if !converts {
            return Err(Report::not_designed(
                span,
                format!("converting `{found}` to `{target_type}` with `as`"),
                "12.3",
            ));
        }
        Ok(RawCall {
            node: Node {
                span,
                kind: NodeKind::IntegerToStr(Box::new(node)),
            },
            value_type: Type::Str,
            failures: Vec::new(),
            unnamed: Some(("what a failed `as` is called", "12.3")),
        })
    }

    /// trace: D44
    fn check_try(&mut self, inner: &Expression, span: Span) -> Outcome<(Node, Type)> {
        if self.context.in_contract {
            return Err(Report::not_designed(span, "`?` in a contract", ""));
        }
        let raw = self.check_raw_call(inner)?;
        if let Some((what, section)) = raw.unnamed {
            return Err(Report::not_designed(span, what, section));
        }
        if !raw.can_fail() {
            return Err(Report::not_designed(
                span,
                "`?` on a call that can't fail",
                "",
            ));
        }
        if self.context.kind == FunctionKind::Main {
            return Err(Report::not_designed(span, "`?` in `fn.main`", "12.6"));
        }
        let Some(listed) = self.context.failures.clone() else {
            return Err(Report::not_designed(
                span,
                "`?` in a function that lists no failures",
                "12.6",
            ));
        };
        for failure in &raw.failures {
            let Some(found) = listed.iter().find(|other| other.full == failure.full) else {
                // trace: D47
                return Err(Report::refused(
                    span,
                    format!(
                        "this passes up `{}`, which this function doesn't list",
                        failure.short
                    ),
                    "`?` is a compile error if the function doesn't list what it passes up (D47)",
                ));
            };
            if found.data != failure.data {
                return Err(Report::not_designed(
                    span,
                    format!(
                        "two failures called `{}` with different data",
                        failure.short
                    ),
                    "",
                ));
            }
        }
        self.check_duties_at_exit(span)?;
        Ok((raw.node, raw.value_type))
    }

    /// trace: D44
    fn check_fallback(
        &mut self,
        value: &Expression,
        fallback: &Expression,
        span: Span,
    ) -> Outcome<(Node, Type)> {
        let raw = self.check_raw_call(value)?;
        if !raw.can_fail() {
            return Err(Report::not_designed(
                span,
                "`??` on a call that can't fail",
                "",
            ));
        }
        // trace: D96
        if matches!(fallback.kind, ExpressionKind::Void) {
            if raw.value_type != Type::Void {
                return Err(Report::not_designed(
                    span,
                    format!("`?? void` on a call that gives back `{}`", raw.value_type),
                    "",
                ));
            }
            return Ok((
                Node {
                    span,
                    kind: NodeKind::Fallback {
                        value: Box::new(raw.node),
                        fallback: Box::new(Node {
                            span: fallback.span,
                            kind: NodeKind::Void,
                        }),
                    },
                },
                Type::Void,
            ));
        }
        if matches!(
            fallback.kind,
            ExpressionKind::Call { .. } | ExpressionKind::Method { .. }
        ) {
            return Err(Report::not_designed(
                fallback.span,
                "whether `??` can take a call, not just a value",
                "12.6",
            ));
        }
        let value_type = raw.value_type.clone();
        let node = self.on_one_path_only(|this| this.check_into(fallback, &value_type))?;
        Ok((
            Node {
                span,
                kind: NodeKind::Fallback {
                    value: Box::new(raw.node),
                    fallback: Box::new(node),
                },
            },
            value_type,
        ))
    }

    /// trace: D45
    fn check_otherwise(
        &mut self,
        value: &Expression,
        handling: &Handling,
        span: Span,
    ) -> Outcome<(Node, Type)> {
        let raw = self.check_raw_call(value)?;
        if let Some((what, section)) = raw.unnamed {
            return Err(Report::not_designed(span, what, section));
        }
        if !raw.can_fail() {
            return Err(Report::not_designed(
                span,
                "`otherwise` on a call that can't fail",
                "",
            ));
        }
        let arms = match handling {
            Handling::Block(arms) => arms.clone(),
            Handling::Named {
                family,
                name,
                jumps,
                span: handler_span,
            } => self.find_handler(family, name, *jumps, *handler_span)?,
        };
        let code = self.check_arms(&arms, &raw.failures, &raw.value_type, span)?;
        Ok((
            Node {
                span,
                kind: NodeKind::Handle {
                    value: Box::new(raw.node),
                    arms: code,
                },
            },
            raw.value_type,
        ))
    }

    /// trace: D46
    fn find_handler(
        &self,
        family: &str,
        name: &str,
        jumps: bool,
        span: Span,
    ) -> Outcome<Vec<crate::ast::Arm>> {
        let Some(arms) = self.handlers.get(&(family.to_string(), name.to_string())) else {
            return Err(Report::refused(
                span,
                format!("there's no handler `{family}.{name}`"),
                "a handler narrowing is a named set of `otherwise` arms (D46)",
            ));
        };
        let can_jump = arms.iter().any(|arm| statements_jump(&arm.body, &[]));
        if can_jump && !jumps {
            return Err(Report::refused(
                span,
                "this handler can jump, so it's marked `*` here",
                "a handler that can `return` or `break` is marked `*` where it's used (D46, D128)",
            )
            .fix(format!("{family}.*{name}")));
        }
        if !can_jump && jumps {
            return Err(Report::refused(
                span,
                "this handler can't jump, so it isn't marked `*`",
                "the compiler refuses `*` on a handler that can't jump (D46)",
            )
            .fix(format!("{family}.{name}")));
        }
        Ok(arms.clone())
    }

    /// trace: D91
    fn check_bake(
        &mut self,
        inner: &Expression,
        span: Span,
        expected: Option<&Type>,
    ) -> Outcome<(Node, Type)> {
        match &inner.kind {
            ExpressionKind::Call { path, .. } => {
                let is_baked = self
                    .signature_of(&path.text)
                    .is_some_and(|signature| signature.kind == FunctionKind::PureBaked);
                if !is_baked {
                    return Err(Report::refused(
                        inner.span,
                        format!("`{}` isn't `fn.pure.baked`", path.text),
                        "`bake` can call normal functions marked `fn.pure.baked`, and only those (D91)",
                    ));
                }
            }
            // `check_own_method` sees the receiver's type, so it refuses one that isn't baked.
            ExpressionKind::Method { .. } => {}
            _ => {
                return Err(Report::not_designed(
                    inner.span,
                    "`bake` on anything but a call",
                    "12.5",
                ));
            }
        }
        self.refuse_if_not_constant(inner)?;
        self.context.in_constant += 1;
        let result = self.check_expression(inner, expected);
        self.context.in_constant -= 1;
        let (node, found) = result?;
        self.compile_time.push(node.clone());
        Ok((
            Node {
                span,
                kind: node.kind,
            },
            found,
        ))
    }

    /// trace: D116
    fn check_negate(
        &mut self,
        inner: &Expression,
        span: Span,
        expected: Option<&Type>,
    ) -> Outcome<(Node, Type)> {
        let hint = number_hint(expected);
        let (node, found) = self.check_expression(inner, hint.as_ref())?;
        let (number, result) = match found {
            Type::Integer { kind, unit, .. } => (
                Number::Integer(kind),
                Type::Integer {
                    kind,
                    bound: None,
                    unit,
                },
            ),
            Type::Float {
                kind, unit, rules, ..
            } => (
                Number::Float(kind, rules),
                Type::Float {
                    kind,
                    bound: None,
                    unit,
                    rules,
                },
            ),
            other => {
                return Err(Report::refused(
                    inner.span,
                    format!("unary `-` negates a number, and this is `{other}`"),
                    "unary `-` negates (D116)",
                ));
            }
        };
        Ok((
            Node {
                span,
                kind: NodeKind::Negate(number, Box::new(node)),
            },
            result,
        ))
    }

    /// Two numbers with one type, where a literal takes its type from the other.
    pub(super) fn check_two_numbers(
        &mut self,
        left: &Expression,
        right: &Expression,
    ) -> Outcome<((Node, Type), (Node, Type))> {
        self.check_operands(left, right, None)
    }

    /// Reads two operands, so a literal takes its type from the other side.
    fn check_operands(
        &mut self,
        left: &Expression,
        right: &Expression,
        expected: Option<&Type>,
    ) -> Outcome<((Node, Type), (Node, Type))> {
        let hint = number_hint(expected);
        if is_number_literal(left) && !is_number_literal(right) {
            let right_checked = self.check_expression(right, hint.as_ref())?;
            let left_hint = number_hint(Some(&right_checked.1));
            let left_checked = self.check_expression(left, left_hint.as_ref())?;
            return Ok((left_checked, right_checked));
        }
        let left_checked = self.check_expression(left, hint.as_ref())?;
        let right_hint = number_hint(Some(&left_checked.1)).or(hint);
        let right_checked = self.check_expression(right, right_hint.as_ref())?;
        Ok((left_checked, right_checked))
    }

    fn check_binary(
        &mut self,
        operator: BinaryOperator,
        left: &Expression,
        right: &Expression,
        span: Span,
        expected: Option<&Type>,
    ) -> Outcome<(Node, Type)> {
        if matches!(
            operator,
            BinaryOperator::And
                | BinaryOperator::Or
                | BinaryOperator::AndBoth
                | BinaryOperator::OrBoth
        ) {
            return self.check_logic(operator, left, right, span);
        }
        // trace: D236
        if self.sides_are_own(left, right) {
            if let Some(word) = super::operators::arithmetic_word(operator) {
                return self.own_arithmetic(word, left, right, span);
            }
        }
        // trace: D249
        for side in [left, right] {
            if self.literal_with_a_split(side) {
                return Err(Report::not_designed(
                    side.span,
                    format!(
                        "whether `{}` in maths is a point or a span",
                        self.text_of(side.span)
                    ),
                    "12.2",
                ));
            }
        }
        let ((left_node, left_type), (right_node, right_type)) =
            self.check_operands(left, right, expected)?;
        let arithmetic = match operator {
            // `and` and `or` left for `check_logic` above.
            BinaryOperator::Add
            | BinaryOperator::And
            | BinaryOperator::Or
            | BinaryOperator::AndBoth
            | BinaryOperator::OrBoth => Arithmetic::Add,
            BinaryOperator::Subtract => Arithmetic::Subtract,
            BinaryOperator::Multiply => Arithmetic::Multiply,
            BinaryOperator::PlusWrap => Arithmetic::PlusWrap,
            BinaryOperator::Divide(narrowing) => Arithmetic::Divide(narrowing),
            BinaryOperator::Remainder(narrowing) => Arithmetic::Remainder(narrowing),
            BinaryOperator::BitAnd => Arithmetic::BitAnd,
            BinaryOperator::BitOr => Arithmetic::BitOr,
            BinaryOperator::BitXor => Arithmetic::BitXor,
            BinaryOperator::ShiftLeft => Arithmetic::ShiftLeft,
            BinaryOperator::ShiftRight => Arithmetic::ShiftRight,
        };
        let (number, result) =
            self.arithmetic_type(arithmetic, &left_type, &right_type, right, span)?;
        Ok((
            Node {
                span,
                kind: NodeKind::Arithmetic {
                    operator: arithmetic,
                    number,
                    left: Box::new(left_node),
                    right: Box::new(right_node),
                },
            },
            result,
        ))
    }

    /// The result type of maths on two operands, with every unit rule. (D100, D111)
    #[allow(clippy::too_many_lines)] // Integers and floats each have their own rules here.
    fn arithmetic_type(
        &self,
        operator: Arithmetic,
        left: &Type,
        right: &Type,
        right_expression: &Expression,
        span: Span,
    ) -> Outcome<(Number, Type)> {
        match (left, right) {
            (
                Type::Integer {
                    kind: left_kind,
                    unit: left_unit,
                    ..
                },
                Type::Integer {
                    kind: right_kind,
                    unit: right_unit,
                    ..
                },
            ) => {
                if left_kind != right_kind {
                    return Err(Report::refused(
                        span,
                        format!("this mixes `{left}` and `{right}`"),
                        WHY_TYPES,
                    ));
                }
                // trace: D48, D164
                if let Arithmetic::Divide(Narrowing::Bare(word)) = operator {
                    return Err(Report::refused(
                        word,
                        "an integer has no bare `div`, so pick `div.trunc` or `div.floor`",
                        "integer division has no single right answer, so you pick one (D48, D164)",
                    ));
                }
                let unit = self.unit_of_result(
                    operator,
                    left_unit.as_ref(),
                    right_unit.as_ref(),
                    left,
                    right,
                    right_expression,
                    span,
                )?;
                Ok((
                    Number::Integer(*left_kind),
                    Type::Integer {
                        kind: *left_kind,
                        bound: None,
                        unit,
                    },
                ))
            }
            (
                Type::Float {
                    kind: left_kind,
                    unit: left_unit,
                    rules: left_rules,
                    ..
                },
                Type::Float {
                    kind: right_kind,
                    unit: right_unit,
                    rules: right_rules,
                    ..
                },
            ) => {
                if left_kind != right_kind {
                    return Err(Report::refused(
                        span,
                        format!("this mixes `{left}` and `{right}`"),
                        WHY_TYPES,
                    ));
                }
                // trace: D159
                if left_rules != right_rules {
                    return Err(float_rules_clash(
                        span,
                        format!("this mixes `{left}` and `{right}`"),
                        left,
                        right,
                    ));
                }
                match operator {
                    // trace: D164
                    Arithmetic::Add
                    | Arithmetic::Subtract
                    | Arithmetic::Multiply
                    | Arithmetic::Divide(Narrowing::Bare(_)) => {}
                    Arithmetic::Divide(narrowing) => {
                        return Err(Report::not_designed(
                            span,
                            format!("`div.{}` on floats", narrowing.word()),
                            "12.2",
                        ));
                    }
                    Arithmetic::Remainder(_) => {
                        return Err(Report::not_designed(span, "`mod` on floats", "12.2"));
                    }
                    Arithmetic::PlusWrap => {
                        return Err(Report::refused(
                            span,
                            "`plus.wrap` is for integers",
                            "a wrapping operator wraps in every mode (D42)",
                        ));
                    }
                    _ => {
                        return Err(Report::refused(
                            span,
                            "bit operators are for integers",
                            "`bit_and`, `bit_or`, `bit_xor`, `bit_not`, `shl` and `shr` are words (D49)",
                        ));
                    }
                }
                let unit = self.unit_of_result(
                    operator,
                    left_unit.as_ref(),
                    right_unit.as_ref(),
                    left,
                    right,
                    right_expression,
                    span,
                )?;
                Ok((
                    Number::Float(*left_kind, *left_rules),
                    Type::Float {
                        kind: *left_kind,
                        bound: None,
                        unit,
                        rules: *left_rules,
                    },
                ))
            }
            // trace: D134
            (Type::New { .. }, _) | (_, Type::New { .. }) => {
                if left == right || is_number_literal(right_expression) {
                    return Err(Report::not_designed(span, "maths on a `type.new`", "12.1"));
                }
                Err(Self::mismatch(span, left, right))
            }
            _ => Err(Report::refused(
                span,
                format!("maths needs numbers, and this has `{left}` and `{right}`"),
                WHY_TYPES,
            )),
        }
    }

    #[allow(clippy::too_many_arguments)] // Each is needed for the message it can give.
    fn unit_of_result(
        &self,
        operator: Arithmetic,
        left_unit: Option<&String>,
        right_unit: Option<&String>,
        left: &Type,
        right: &Type,
        right_expression: &Expression,
        span: Span,
    ) -> Outcome<Option<String>> {
        match operator {
            Arithmetic::Add | Arithmetic::Subtract | Arithmetic::PlusWrap => {
                if let Some(result) =
                    Self::point_and_span(operator, left_unit, right_unit, left, right, span)?
                {
                    return Ok(Some(result));
                }
                if self.same_unit(left_unit, right_unit) {
                    return Ok(left_unit.cloned());
                }
                // trace: D101
                if let (Some(unit), None, Some(_)) =
                    (left_unit, right_unit, literal_integer(right_expression))
                {
                    let shown = self.text_of(right_expression.span);
                    return Err(Report::refused(
                        right_expression.span,
                        format!("this adds a plain number to `{left}`"),
                        "the compiler checks units, and a unit is required where the type asks for one (D100, D101)",
                    )
                    .fix(format!("{shown} {unit}")));
                }
                // trace: D100
                Err(Report::refused(
                    span,
                    format!("this mixes `{left}` and `{right}`"),
                    "the same units can add and compare, and mixing them is refused (D100)",
                ))
            }
            Arithmetic::Multiply | Arithmetic::Divide(_) | Arithmetic::Remainder(_) => {
                if left_unit.is_none() || right_unit.is_none() {
                    if left_unit.is_none() && right_unit.is_none() {
                        return Ok(None);
                    }
                    return Err(Report::not_designed(
                        span,
                        "scaling a unit by a plain number, and what unit comes back",
                        "",
                    ));
                }
                if matches!(operator, Arithmetic::Remainder(_)) {
                    return Err(Report::not_designed(span, "a remainder of two units", ""));
                }
                // Units multiply and divide as powers, and storing needs a declared unit
                // whose powers match. (D242)
                // trace: D242
                self.combined_unit(
                    left_unit,
                    right_unit,
                    operator != Arithmetic::Multiply,
                    span,
                )
            }
            _ => {
                if left_unit.is_none() && right_unit.is_none() {
                    Ok(None)
                } else {
                    Err(Report::not_designed(
                        span,
                        "bit operators on a number with a unit",
                        "",
                    ))
                }
            }
        }
    }

    fn check_logic(
        &mut self,
        operator: BinaryOperator,
        left: &Expression,
        right: &Expression,
        span: Span,
    ) -> Outcome<(Node, Type)> {
        let left_node = self.check_condition(left)?;
        // trace: D124
        let right_node = if matches!(operator, BinaryOperator::And | BinaryOperator::Or) {
            // The right side only runs if it's needed, so what it gives away is only maybe gone.
            self.on_one_path_only(|this| this.check_condition(right))?
        } else {
            self.check_condition(right)?
        };
        let (left_node, right_node) = (Box::new(left_node), Box::new(right_node));
        let kind = match operator {
            BinaryOperator::And => NodeKind::And(left_node, right_node),
            BinaryOperator::Or => NodeKind::Or(left_node, right_node),
            BinaryOperator::AndBoth => NodeKind::AndBoth(left_node, right_node),
            _ => NodeKind::OrBoth(left_node, right_node),
        };
        Ok((Node { span, kind }, Type::Bool))
    }

    /// trace: D117
    fn check_compare(
        &mut self,
        first: &Expression,
        rest: &[(CompareOperator, Expression)],
        span: Span,
    ) -> Outcome<(Node, Type)> {
        // Whether a struct has `==` without writing `op ==` isn't decided, so with no `op ==`
        // that stays as it was. (D237)
        // trace: D237
        if rest.first().is_some_and(|(operator, right)| {
            self.sides_are_own(first, right)
                && (!matches!(operator, CompareOperator::Equal | CompareOperator::NotEqual)
                    || self.has_operator("==", first, right)
                    || self.is_generic(first)
                    || self.is_generic(right))
        }) {
            return self.own_comparison_chain(first, rest, span);
        }
        let mut nodes = Vec::new();
        let mut previous: Option<Type> = None;
        let mut first_node = None;
        let mut left = first;
        for (operator, right) in rest {
            let (left_checked, right_checked) = if let Some(left_type) = &previous {
                let hint = number_hint(Some(left_type));
                // trace: D124
                let right_checked =
                    self.on_one_path_only(|this| this.check_expression(right, hint.as_ref()))?;
                ((None, left_type.clone()), right_checked)
            } else {
                let ((left_node, left_type), right_checked) =
                    self.check_operands(left, right, None)?;
                ((Some(left_node), left_type), right_checked)
            };
            let (left_node, left_type) = left_checked;
            let (right_node, right_type) = right_checked;
            self.check_comparable(*operator, &left_type, &right_type, left, right)?;
            if let Some(left_node) = left_node {
                first_node = Some(left_node);
            }
            nodes.push((*operator, right_node));
            previous = Some(right_type);
            left = right;
        }
        let Some(first_node) = first_node else {
            return Err(Report::refused(span, "this comparison is empty", "(D49)"));
        };
        Ok((
            Node {
                span,
                kind: NodeKind::Compare {
                    first: Box::new(first_node),
                    rest: nodes,
                },
            },
            Type::Bool,
        ))
    }

    #[allow(clippy::too_many_lines)] // Each kind of comparison has its own message.
    fn check_comparable(
        &self,
        operator: CompareOperator,
        left: &Type,
        right: &Type,
        left_expression: &Expression,
        right_expression: &Expression,
    ) -> Outcome<()> {
        let span = left_expression.span.to(right_expression.span);
        match (left, right) {
            (
                Type::Integer {
                    kind: left_kind,
                    unit: left_unit,
                    ..
                },
                Type::Integer {
                    kind: right_kind,
                    unit: right_unit,
                    ..
                },
            ) => {
                if left_kind != right_kind {
                    return Err(Report::refused(
                        span,
                        format!("this compares `{left}` and `{right}`"),
                        WHY_TYPES,
                    ));
                }
                if left_unit != right_unit {
                    // trace: D101
                    let plain_literal = match (left_unit, right_unit) {
                        (Some(unit), None) if literal_integer(right_expression).is_some() => {
                            Some((right_expression, unit))
                        }
                        (None, Some(unit)) if literal_integer(left_expression).is_some() => {
                            Some((left_expression, unit))
                        }
                        _ => None,
                    };
                    if let Some((literal, unit)) = plain_literal {
                        let shown = self.text_of(literal.span);
                        return Err(Report::refused(
                            literal.span,
                            format!("this compares a plain number with `{unit}`"),
                            "the compiler checks units, and a unit is required where the type asks for one (D100, D101)",
                        )
                        .fix(format!("{shown} {unit}")));
                    }
                    // trace: D100
                    return Err(Report::refused(
                        span,
                        format!("this compares `{left}` and `{right}`"),
                        "the same units can add and compare, and mixing them is refused (D100)",
                    ));
                }
                Ok(())
            }
            (
                Type::Float {
                    kind: left_kind,
                    unit: left_unit,
                    rules: left_rules,
                    ..
                },
                Type::Float {
                    kind: right_kind,
                    unit: right_unit,
                    rules: right_rules,
                    ..
                },
            ) => {
                if left_kind != right_kind || left_unit != right_unit {
                    return Err(Report::refused(
                        span,
                        format!("this compares `{left}` and `{right}`"),
                        WHY_TYPES,
                    ));
                }
                // trace: D159
                if left_rules != right_rules {
                    return Err(float_rules_clash(
                        span,
                        format!("this compares `{left}` and `{right}`"),
                        left,
                        right,
                    ));
                }
                // trace: D160
                Ok(())
            }
            (Type::Bool, Type::Bool) => {
                if matches!(operator, CompareOperator::Equal | CompareOperator::NotEqual) {
                    Ok(())
                } else {
                    Err(Report::not_designed(
                        span,
                        format!("`{}` on `bool`", operator.word()),
                        "",
                    ))
                }
            }
            // trace: D134
            (Type::New { .. }, _) | (_, Type::New { .. })
                if left != right
                    && !is_number_literal(left_expression)
                    && !is_number_literal(right_expression) =>
            {
                Err(Self::mismatch(span, left, right))
            }
            (Type::New { .. }, _) | (_, Type::New { .. }) if left != right => Err(
                Report::not_designed(span, "comparing a `type.new` with a literal", "12.1"),
            ),
            (left_type, right_type) if left_type == right_type => Err(Report::not_designed(
                span,
                format!("comparing `{left_type}` values"),
                "",
            )),
            _ => Err(Report::refused(
                span,
                format!("this compares `{left}` and `{right}`"),
                WHY_TYPES,
            )),
        }
    }
}

/// Whether a handler's arm can `return` or `break` out of itself, which is what the `*` marks.
/// A `break` of a loop inside the arm stays inside it. (D46, D128)
fn statements_jump(statements: &[crate::ast::Statement], loops: &[Option<&str>]) -> bool {
    use crate::ast::StatementKind;
    let in_a_loop = |body: &[crate::ast::Statement], name: Option<&crate::ast::Name>| {
        let mut around = loops.to_vec();
        around.push(name.map(|name| name.text.as_str()));
        statements_jump(body, &around)
    };
    statements.iter().any(|statement| match &statement.kind {
        StatementKind::Return(_) => true,
        StatementKind::Break { name: None } => loops.is_empty(),
        StatementKind::Break { name: Some(path) } => {
            let wanted = path.text.strip_prefix("my::").unwrap_or(&path.text);
            !loops.contains(&Some(wanted))
        }
        StatementKind::If { links, last } => {
            links.iter().any(|link| statements_jump(&link.body, loops))
                || last
                    .as_ref()
                    .is_some_and(|block| statements_jump(block, loops))
        }
        StatementKind::Shield { body, .. } => statements_jump(body, loops),
        StatementKind::For {
            loop_name, body, ..
        }
        | StatementKind::While {
            loop_name, body, ..
        }
        | StatementKind::Forever { loop_name, body } => in_a_loop(body, loop_name.as_ref()),
        StatementKind::Match { arms, .. } => {
            arms.iter().any(|arm| statements_jump(&arm.body, loops))
        }
        _ => false,
    })
}

/// What's wrong with how an `inline` path is written, if anything, and its fix. Inline assembly
/// is base, as `inline.asm`, and a limb's language is `inline.@limb::lang`. (D227)
/// trace: D227
pub(super) fn inline_spelling(text: &str, span: Span) -> Option<Box<Report>> {
    const WHY: &str = "inline code is `inline.asm`, in the base language, or a limb's own language as `inline.@limb::lang` (D227)";
    if text == "@std::inline.asm" {
        return Some(
            Report::refused(span, "inline assembly is in the base language", WHY).fix("inline.asm"),
        );
    }
    let language = text.strip_prefix("inline.")?;
    if language == "asm" || language.starts_with('@') {
        return None;
    }
    if language.contains("::") {
        return Some(
            Report::refused(span, "a limb's inline language says its limb with `@`", WHY)
                .fix(format!("inline.@{language}")),
        );
    }
    Some(Report::refused(
        span,
        format!("`{text}` isn't an inline language"),
        WHY,
    ))
}

impl Checker<'_> {
    /// An `inline` call: spelled right, inside a shield, and then not designed, since how the
    /// code inside is written isn't. (D227, D38)
    /// trace: D227, D38
    fn inline_call(&self, text: &str, span: Span) -> Option<Box<Report>> {
        if let Some(report) = inline_spelling(text, span) {
            return Some(report);
        }
        // trace: D233
        if text == "inline.asm" {
            return Some(Report::refused(
                span,
                "assembly is a block, `inline.asm:`, with its `reads:`, `writes:` and `clobbers:` lines",
                "an assembly block is a statement that says what goes in and out, then holds raw assembly (D233)",
            ));
        }
        if !text.starts_with("inline.@") {
            return None;
        }
        if self.context.in_shield == 0 {
            return Some(Report::refused(
                span,
                format!("`{text}` must be inside a shield"),
                "a shield is the only place dangerous code may live (D34, D38, D227)",
            ));
        }
        let language = text.strip_prefix("inline.@").unwrap_or(text);
        Some(Report::not_designed(
            span,
            format!(
                "how the limb `@{}` is found",
                language.split("::").next().unwrap_or(language)
            ),
            "12.9",
        ))
    }
}

impl Checker<'_> {
    /// A value's unit the way this file writes it: by the short name a `use` gives it, or else
    /// by its family and name with its package path left off. (D246)
    /// trace: D246
    fn unit_as_written(&self, found: &Type, span: Span) -> Outcome<String> {
        let (Type::Integer {
            unit: Some(unit), ..
        }
        | Type::Float {
            unit: Some(unit), ..
        }) = found
        else {
            return Err(Report::refused(
                span,
                format!("`{found}` has no unit"),
                "(D246)",
            ));
        };
        // A point or span prints its unit alone, as `20.0 celsius`. (D249)
        let unit = &super::units::split_of(unit).0.to_string();
        let mut shortened: Vec<&String> = self
            .aliases
            .iter()
            .filter(|(_, path)| **path == *unit || path.strip_prefix("my::") == Some(unit.as_str()))
            .map(|(short, _)| short)
            .collect();
        shortened.sort();
        if let Some(short) = shortened.first() {
            return Ok((*short).clone());
        }
        Ok(unit
            .rsplit_once("::")
            .map_or(unit.as_str(), |(_, rest)| rest)
            .to_string())
    }
}

impl Checker<'_> {
    /// A format's pieces, each hole with its value's slot, its shapes checked, and its unit
    /// written the way this file writes it when it asks for one. (D194, D246)
    fn print_pieces(
        &self,
        pieces: Vec<crate::ast::FormatPiece>,
        slots: &[usize],
        value_types: &[TypedSpan],
        stored: bool,
    ) -> Outcome<Vec<ir::PrintPiece>> {
        let mut printed = Vec::new();
        let mut holes_seen = 0;
        for piece in pieces {
            match piece {
                crate::ast::FormatPiece::Text(text) => printed.push(ir::PrintPiece::Text(text)),
                crate::ast::FormatPiece::Hole(hole) => {
                    let value = slots.get(holes_seen).copied().unwrap_or_default();
                    holes_seen += 1;
                    let mut shapes = hole.shapes;
                    if let Some((found, value_span)) = value_types.get(value) {
                        Self::check_shapes(found, &shapes, *value_span, stored)?;
                        // trace: D246
                        if shapes.unit {
                            shapes.unit_text = Some(self.unit_as_written(found, *value_span)?);
                        }
                    }
                    printed.push(ir::PrintPiece::Hole {
                        value,
                        shapes,
                        span: hole.span,
                    });
                }
            }
        }
        Ok(printed)
    }
}

impl Checker<'_> {
    /// `std::math::sin`, `cos` and `tan` ask for an angle in radians, so a plain number is
    /// refused, asking for one, and their answer is correctly rounded. (D245, D256, D161)
    /// trace: D245, D256
    fn check_trigonometry(
        &mut self,
        function: &str,
        arguments: &[Argument],
        span: Span,
        path_span: Span,
    ) -> Outcome<RawCall> {
        const RADIANS: &str = "std::units::angle.radians";
        const WHY: &str = "trigonometry asks for an angle in radians, so a plain number is refused, and degrees are converted with `as` (D245)";
        let [argument] = arguments else {
            return Err(Report::refused(
                path_span,
                format!("`std::math::{function}` takes one angle"),
                "trigonometry asks for an angle in radians (D245)",
            ));
        };
        if is_number_literal(&argument.value) {
            return Err(Report::refused(
                argument.value.span,
                format!(
                    "this is a plain number, where an angle, `f64.stable.{RADIANS}`, is needed"
                ),
                WHY,
            ));
        }
        let (node, found) = self.check_expression(&argument.value, None)?;
        let Type::Float {
            kind: FloatKind::F64,
            unit: Some(unit),
            rules,
            ..
        } = &found
        else {
            return Err(Report::refused(
                argument.value.span,
                format!("this is `{found}`, where an angle, `f64.stable.{RADIANS}`, is needed"),
                WHY,
            ));
        };
        if unit != RADIANS {
            return Err(Report::refused(
                argument.value.span,
                format!("this is `{found}`, where an angle, `f64.stable.{RADIANS}`, is needed"),
                WHY,
            ));
        }
        if *rules != FloatRules::Plain {
            return Err(Report::not_designed(
                argument.value.span,
                format!("`std::math::{function}` of `{found}`"),
                "",
            ));
        }
        // Correctly rounded, worked out by the oracle itself. (D256, D161)
        // trace: D256, D161
        let function = match function {
            "sin" => crate::maths::Function::Sin,
            "cos" => crate::maths::Function::Cos,
            _ => crate::maths::Function::Tan,
        };
        Ok(RawCall {
            node: Node {
                span,
                kind: NodeKind::Maths {
                    function,
                    value: Box::new(node),
                },
            },
            value_type: Type::Float {
                kind: FloatKind::F64,
                bound: None,
                unit: None,
                rules: FloatRules::Plain,
            },
            failures: Vec::new(),
            unnamed: None,
        })
    }
}
