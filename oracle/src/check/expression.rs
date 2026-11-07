//! Expressions: what each form means, its type, and the checks hidden inside it.

use super::body::root_variable;
use super::hand::HandedOff;
use super::{Checker, Failure, Origin, ParameterInfo, Signature, WHY_TYPES};
use crate::ast::{
    Argument, BinaryOperator, CompareOperator, Expression, ExpressionKind, FieldValue,
    FunctionKind, HandWord, Handling, ParameterWord,
};
use crate::ir::{self, Arithmetic, Node, NodeKind, Number};
use crate::report::{Outcome, Report, ReportExtra, Span};
use crate::syntax::is_clashing;
use crate::types::{FloatKind, IntegerKind, Type, Value};

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
            Type::Float { kind, unit: from },
            Type::Float {
                kind: to_kind,
                unit: to,
            },
        ) => kind == to_kind && from.is_some() != to.is_some(),
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

fn number_hint(value_type: Option<&Type>) -> Option<Type> {
    match value_type {
        Some(Type::Integer { kind, .. }) => Some(Type::plain_integer(*kind)),
        Some(Type::Float { kind, .. }) => Some(Type::Float {
            kind: *kind,
            unit: None,
        }),
        _ => None,
    }
}

/// Whether `text` in decimal is exactly what `value` holds. (D59)
fn decimal_is_exact(text: &str, value: f64) -> bool {
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
            if kind == target_kind && unit == target_unit {
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
            },
            Type::Float {
                kind: target_kind,
                unit: target_unit,
            },
        ) = (found, target)
        {
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
    fn refuse_if_not_constant(&self, value: &Expression) -> Outcome<()> {
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
                let resolved = self.resolve_unit(unit)?;
                if matches!(number.kind, ExpressionKind::Float { .. }) {
                    Self::check_float_literal(number, expected, Some(resolved))
                } else {
                    Self::check_integer_literal(number, expected, Some(resolved))
                }
            }
            ExpressionKind::Text(text) => Ok((
                simple(NodeKind::Constant(Value::Str(text.clone()))),
                Type::Str,
            )),
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
            ExpressionKind::Path(path) => self.check_path_value(path, span),
            ExpressionKind::Build { path, fields } => self.check_build(path, fields, span),
            ExpressionKind::Call { .. } | ExpressionKind::Method { .. } => {
                let raw = self.check_raw_call(expression)?;
                Ok(Self::crash_if_it_fails(raw, span))
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
            ExpressionKind::As { value, target } => {
                let raw = self.check_as(value, target, span)?;
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
    fn check_float_literal(
        expression: &Expression,
        expected: Option<&Type>,
        unit: Option<String>,
    ) -> Outcome<(Node, Type)> {
        let span = expression.span;
        let ExpressionKind::Float {
            digits,
            negative,
            inexact,
        } = &expression.kind
        else {
            return Err(Report::refused(span, "this number can't be read", "(D59)"));
        };
        let kind = match expected {
            Some(Type::Float { kind, .. }) => *kind,
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
            return Err(Report::not_designed(
                span,
                format!("a literal too large for `{}`", kind.word()),
                "",
            ));
        }
        let exact = decimal_is_exact(digits, held);
        let shown = format!("{}{digits}", if *negative { "-" } else { "" });
        if exact && *inexact {
            return Err(Report::refused(
                span,
                "this literal is exact, so it takes no `~`",
                format!("{} holds {shown} exactly (D59)", kind.word()),
            )
            .fix(shown));
        }
        if !exact && !*inexact {
            return Err(Report::refused(
                span,
                "this literal gets rounded, so it says so",
                format!("{} can't hold {shown} exactly (D59)", kind.word()),
            )
            .fix(format!("~{shown}")));
        }
        Ok((
            Node {
                span,
                kind: NodeKind::Constant(value),
            },
            Type::Float { kind, unit },
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
            self.require_std(span)?;
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
            let field_type = declared
                .get(index)
                .map_or(Type::Void, |(_, field_type)| field_type.clone());
            nodes.push((index, self.check_into(&field.value, &field_type)?));
        }
        Ok(nodes)
    }

    fn check_build(
        &mut self,
        path: &crate::ast::Path,
        fields: &[FieldValue],
        span: Span,
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
        if let Some(info) = self.structs.get(&bare).cloned() {
            let nodes = self.check_field_values(&info.fields, fields, &bare, span)?;
            return Ok((
                Node {
                    span,
                    kind: NodeKind::Struct(nodes),
                },
                Type::Struct(bare),
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
                let nodes = self.check_field_values(&declared, fields, &bare, span)?;
                return Ok((
                    Node {
                        span,
                        kind: NodeKind::Case(case, nodes),
                    },
                    Type::Union(union_name.to_string()),
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
                arguments,
            } => self.check_method(receiver, kit.as_deref(), name, *percent, arguments, span),
            ExpressionKind::As { value, target } => self.check_as(value, target, span),
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
        if path == "std::println" {
            return Err(Report::refused(
                span,
                "`println` is in `std`'s kit part",
                "`@` goes only on things a limb provides (D22)",
            )
            .fix("std::println"));
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

    fn check_call(
        &mut self,
        path: &crate::ast::Path,
        arguments: &[Argument],
        span: Span,
    ) -> Outcome<RawCall> {
        let text = self.canonical(&path.text);
        self.refuse_limb_outside_shield(&text, path.span)?;
        if let Some(rest) = text.strip_prefix("std::") {
            self.require_std(path.span)?;
            return self.check_std_call(rest, arguments, span, path.span);
        }
        let bare = match text.strip_prefix("my::") {
            Some(mine) => {
                if !is_clashing(mine) {
                    return Err(Report::not_designed(
                        path.span,
                        format!("`my::` on `{mine}`, which doesn't clash"),
                        "",
                    ));
                }
                mine.to_string()
            }
            None => text.clone(),
        };
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
        Ok(RawCall {
            node: Node {
                span,
                kind: NodeKind::Call {
                    function: signature.index,
                    arguments: code,
                },
            },
            value_type: signature.return_type.clone(),
            failures: signature.failures.clone(),
            unnamed: None,
        })
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

    /// trace: D95
    fn check_std_call(
        &mut self,
        rest: &str,
        arguments: &[Argument],
        span: Span,
        path_span: Span,
    ) -> Outcome<RawCall> {
        if rest.starts_with("println.") {
            // trace: D51
            return Err(Report::refused(
                path_span,
                format!("`std::{rest}` spells out what bare `std::println` means"),
                "a default `std` fixes can't be written out (D51)",
            )
            .fix("std::println"));
        }
        if rest != "println" {
            return Err(Report::not_designed(
                path_span,
                format!("`std::{rest}`, since `std`'s full contents aren't chosen"),
                "",
            ));
        }
        if self.is_pure() || self.context.in_contract {
            // trace: D86
            return Err(Report::refused(
                path_span,
                "printing is I/O, which a pure function refuses",
                "a pure function refuses I/O, and contracts can only call pure functions (D86)",
            ));
        }
        if arguments.is_empty() {
            return Err(Report::not_designed(
                span,
                "`std::println` with no pieces",
                "",
            ));
        }
        let piece = ParameterInfo {
            name: "piece".to_string(),
            label: None,
            word: ParameterWord::Bare,
            parameter_type: Type::Str,
            default: None,
            label_has_unit: false,
        };
        let mut handed: Vec<HandedOff> = Vec::new();
        let mut nodes = Vec::new();
        for argument in arguments {
            // trace: D114
            if let Some(label) = &argument.label {
                return Err(Report::refused(
                    label.span,
                    "`print`'s pieces have no labels",
                    "`std` labels every other argument, except lists where every item has the same role, like `print`'s pieces (D114)",
                ));
            }
            // A number gets `as str` to be printed, and that's the one fix.
            let value = &argument.value;
            let code = self.check_println_piece(value, &piece, &mut handed)?;
            nodes.push(code);
        }
        Ok(RawCall {
            node: Node {
                span,
                kind: NodeKind::Println(nodes),
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

    fn check_println_piece(
        &mut self,
        value: &Expression,
        piece: &ParameterInfo,
        handed: &mut Vec<HandedOff>,
    ) -> Outcome<Node> {
        let argument = self.check_argument(value, piece, handed);
        match argument {
            Ok(ir::Argument::Value(node)) => Ok(node),
            Ok(ir::Argument::Alias(place)) => Ok(Node {
                span: value.span,
                kind: NodeKind::Read(place),
            }),
            Err(report) => Err(report),
        }
    }

    /// `$xs.std::%append(val: 4)` is the one method that's decided. (D4, D115)
    #[allow(clippy::too_many_arguments)] // A method call has this many parts.
    #[allow(clippy::too_many_lines)] // Every way to spell a method wrong, top to bottom.
    fn check_method(
        &mut self,
        receiver: &Expression,
        kit: Option<&str>,
        name: &crate::ast::Name,
        percent: bool,
        arguments: &[Argument],
        span: Span,
    ) -> Outcome<RawCall> {
        let receiver_text = self.text_of(receiver.span).to_string();
        let call_text = self.text_of(span);
        let after_name = call_text
            .split_once(&name.text)
            .map_or("", |(_, rest)| rest)
            .to_string();
        let correct = format!("{receiver_text}.std::%{}{after_name}", name.text);
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
        let own_method = match &receiver_type {
            Type::Struct(type_name)
            | Type::Union(type_name)
            | Type::New {
                name: type_name, ..
            } => Some(format!("{type_name}.%{}", name.text)),
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
            return Err(if is_own {
                let mut fixed = report.fix(format!("{receiver_text}.%{}{after_name}", name.text));
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
        self.require_std(span)?;
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
                },
            },
            value_type: signature.return_type.clone(),
            failures: signature.failures.clone(),
            unnamed: None,
        })
    }

    /// trace: D7
    fn check_as(
        &mut self,
        value: &Expression,
        target: &crate::ast::TypeExpression,
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
        ) {
            return Err(Report::refused(
                value.span,
                "this `as` is on more than a single value, so it needs `( )`",
                "`as` works on a single value, and where maths gives no order you group with `( )` (D116)",
            ));
        }
        let target_type = self.resolve_type(target)?;
        // A literal takes its type from where it lands, even here. (D141)
        let (node, found) = if is_number_literal(value) {
            self.check_expression(value, Some(&target_type))?
        } else {
            self.check_expression(value, None)?
        };
        // trace: D141
        if crosses(&found, &target_type) {
            if !found.is_always_copied() && root_variable(value).is_some() {
                return Err(Report::not_designed(
                    value.span,
                    format!("crossing a whole `{found}` that lives in a variable, and what happens to the original"),
                    "",
                ));
            }
            return Ok(RawCall {
                node,
                value_type: target_type,
                failures: Vec::new(),
                unnamed: None,
            });
        }
        // trace: D83
        if self.context.in_constant > 0 {
            return Err(Report::refused(
                span,
                "this `as` isn't known at compile time",
                "a `const`'s value must be fully known at compile time (D83, D91)",
            ));
        }
        if self.is_pure() {
            return Err(Report::not_designed(
                span,
                "whether a `fn.pure` may allocate, as `as` can",
                "12.5",
            ));
        }
        // trace: D122
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
            Type::Float { kind, unit } => (Number::Float(kind), Type::Float { kind, unit }),
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
                },
                Type::Float {
                    kind: right_kind,
                    unit: right_unit,
                },
            ) => {
                if left_kind != right_kind {
                    return Err(Report::refused(
                        span,
                        format!("this mixes `{left}` and `{right}`"),
                        WHY_TYPES,
                    ));
                }
                match operator {
                    Arithmetic::Add | Arithmetic::Subtract | Arithmetic::Multiply => {}
                    Arithmetic::Divide(_) | Arithmetic::Remainder(_) => {
                        return Err(Report::not_designed(span, "dividing floats", "12.2"));
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
                    Number::Float(*left_kind),
                    Type::Float {
                        kind: *left_kind,
                        unit,
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
                if left_unit == right_unit {
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
                let (Some(left_name), Some(right_name)) = (left_unit, right_unit) else {
                    if left_unit.is_none() && right_unit.is_none() {
                        return Ok(None);
                    }
                    return Err(Report::not_designed(
                        span,
                        "scaling a unit by a plain number, and what unit comes back",
                        "",
                    ));
                };
                if matches!(operator, Arithmetic::Remainder(_)) {
                    return Err(Report::not_designed(span, "a remainder of two units", ""));
                }
                let multiply = operator == Arithmetic::Multiply;
                // trace: D111
                let found = self.together.iter().find(|line| {
                    line.multiply == multiply
                        && line.left == *left_name
                        && line.right == *right_name
                });
                match found {
                    Some(line) => Ok(Some(line.result.clone())),
                    None => Err(Report::refused(
                        span,
                        format!("no `together` line says what `{left_name}` and `{right_name}` make"),
                        "multiplying or dividing two units only works where a `together` line declares it (D111)",
                    )),
                }
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
                },
                Type::Float {
                    kind: right_kind,
                    unit: right_unit,
                },
            ) => {
                if left_kind != right_kind || left_unit != right_unit {
                    return Err(Report::refused(
                        span,
                        format!("this compares `{left}` and `{right}`"),
                        WHY_TYPES,
                    ));
                }
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
