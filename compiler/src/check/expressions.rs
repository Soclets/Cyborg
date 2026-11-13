//! Expressions: their types, what they compute, and the runtime checks that stay with them (standard 10.4).

use crate::checked::{self, Arithmetic, Comparison, ExpressionKind as Kind, MachineType};
use crate::diagnostic::{Diagnostic, Kind as DiagnosticKind};
use crate::source::Span;
use crate::syntax::{self, BinaryOperator, ExpressionKind, UnaryOperator};

use super::types::{Bounds, IntegerKind, StrSize, Type};
use super::{Body, Check, Checker, Origin, Reported, State};

use super::names::NEW_WHY;

/// Something handed off in the call being checked: its variable, field path, argument, and whether nothing else
/// may use it meanwhile.
type Handed = (String, Vec<usize>, Span, bool);

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
    let when_template = when
        .iter()
        .map(|part| match part {
            When::Text(text) => checked::WhenPart::Text(text.clone()),
            When::Operand(index) => checked::WhenPart::Operand(*index),
        })
        .collect();
    checked::CheckFailure::rendered(checker.sources, diagnostic, when_template, Vec::new())
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
                Ok(Typed::new(Type::Str(StrSize::View), Kind::Text(text.unwrap_or_default())))
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
                if let Some(typed) = self.build(body, expression, path, labels, expected)? {
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
            // A range is only decided straight after `in`, so it's caught before an item's brackets (D128).
            ExpressionKind::Bracket { base, .. } if matches!(&base.kind, ExpressionKind::Path(path) if path.is_single("range")) => {
                Err(self.error(Diagnostic::not_designed(
                    base.span,
                    "a range anywhere but straight after `in`",
                    NOT_DECIDED,
                )))
            }
            ExpressionKind::Bracket { base, labels, .. } => self.item_value(body, expression, base, labels),
            ExpressionKind::Bake(inner) => {
                self.known_at_compile_time(
                    body,
                    inner,
                    "`bake` can call normal functions marked `fn.pure.baked`, and only those (D91)",
                    false,
                )?;
                let typed = self.expression(body, inner, expected)?;
                let value = self.run_at_compile_time(body, &typed.value, inner.span)?;
                let machine_type = typed.value.machine_type.clone();
                Ok(Typed::new(typed.value_type, value.expression(machine_type).kind))
            }
            ExpressionKind::MethodCall { receiver, method, arguments, .. } => {
                self.method_call(body, expression, receiver, method, arguments)
            }
            ExpressionKind::WithUnit { value, unit } => self.with_unit(body, expression, value, unit, expected),
            // A builtin looks like a call and can't be written as a Cyborg function, and none are chosen yet (D56).
            ExpressionKind::Builtin { name, .. } => {
                let bare = name.text.trim_start_matches('@');
                if matches!(bare, "let" | "fn" | "if" | "for" | "as" | "return" | "struct" | "import" | "mut") {
                    return Err(self.error(Diagnostic::refused(
                        name.span,
                        format!("`@{bare}` can't happen"),
                        "a builtin looks like a function call and can't be written as a Cyborg function, so `let`, `+` and `as` aren't builtins (D56)",
                    )));
                }
                Err(self.error(Diagnostic::not_designed(
                    name.span,
                    format!("the builtin `@{bare}`"),
                    "not decided yet (standard 12.9)",
                )))
            }
            ExpressionKind::Owned { ownership, ownership_span, value } => {
                self.owned(body, *ownership, *ownership_span, value, expected)
            }
            ExpressionKind::As { value, keyword, written_type } => {
                self.conversion(body, expression, value, keyword, written_type)
            }
            ExpressionKind::Call { callee, arguments, .. } => {
                let outer = std::mem::replace(&mut self.call_expected, expected.cloned());
                let result = self.call(body, expression, callee, arguments);
                self.call_expected = outer;
                result
            }
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
        let mut typed = Typed::new(Type::Integer { kind, bounds: None, unit: None }, Kind::Integer(value));
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
                let value_type =
                    Type::Float { bits: 64, kind: super::types::FloatKind::Stable, bounds: None, unit: None };
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
        let value_type = Type::Float { bits, kind, bounds: None, unit: None };
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
        // `as.trunc`, `as.floor` and `as.exact` say how an integer rounds going to a bigger unit (D255).
        let narrowing = match keyword.segments.as_slice() {
            [_] => None,
            [_, word] if matches!(word.name.text.as_str(), "trunc" | "floor" | "exact") => {
                Some((word.name.text.clone(), word.name.span))
            }
            _ => return Err(self.not_yet(keyword.span, format!("`{}`", keyword.text()))),
        };
        if let ExpressionKind::Group(grouped) = &value.kind
            && let ExpressionKind::Owned { ownership, .. } = &grouped.kind
        {
            return Err(self.error(Diagnostic::refused(
                grouped.span,
                format!("`{}` goes where a value is handed off", ownership.word()),
                "the words go on a call's argument, or into a name, a `return` or a field (D2, D3)",
            )));
        }
        let handing = if matches!(value.kind, ExpressionKind::As { .. }) {
            body.handing
        } else {
            std::mem::replace(&mut body.handing, false)
        };
        let typed = self.expression(body, value, None)?;
        let into = self.resolve_type(written_type)?;
        // A crossing of a whole value in a variable says its ownership (D149).
        let crossing = match (&typed.value_type, &into) {
            (Type::New { inner, .. }, to) => **inner == *to || inner.plain() == *to,
            (from, Type::New { inner, .. }) => *from == **inner || from.plain() == **inner,
            _ => false,
        };
        if crossing && !handing && is_held(value) && !typed.value_type.is_copied() {
            let shown = self.slice(value.span).to_string();
            return Err(self.error(Diagnostic::refused(
                value.span,
                format!("`{shown}` is `{}`, so this says `Give` or `Copy`", typed.value_type),
                "`as` on a whole value says its ownership: `Give` leaves it dead, and `Copy` pays for a copy (D3, D149)",
            )));
        }
        let undecided = |from: &Type, into: &Type| {
            Diagnostic::not_designed(
                expression.span,
                format!("converting `{from}` to `{into}` with `as`"),
                "not decided yet (standard 12.3)",
            )
        };
        if let Some((word, span)) = &narrowing {
            let unit_change = typed.value_type.unit().is_some()
                && into.unit().is_some()
                && typed.value_type.with_unit(None).plain() == into.with_unit(None).plain();
            if !unit_change {
                return Err(self.error(Diagnostic::not_designed(
                    *span,
                    format!("`as.{word}` on anything but a change of unit"),
                    "not decided yet (standard 12.2)",
                )));
            }
            if matches!(typed.value_type, Type::Float { .. }) {
                return Err(self.error(Diagnostic::refused(
                    *span,
                    "a float rounds to the nearest, so its `as` says no narrowing",
                    "a value changes unit with `as`: a float rounds to the nearest, an integer going to a smaller unit is exact, and one going to a bigger unit says how it rounds with `as.trunc`, `as.floor` or `as.exact` (D255)",
                )));
            }
        }
        self.narrowing = narrowing.as_ref().map(|(word, _)| word.clone());
        let converted = self.unit_conversion(expression, &typed, &into);
        self.narrowing = None;
        if let Some(converted) = converted {
            return converted;
        }
        match (&typed.value_type, &into) {
            (Type::Unknown, _) => Err(Reported),
            // Crossing in or out of a `type.new` is the same data, so it can't fail (D141). One over a bound only goes
            // out, since going in would have to check it.
            (Type::New { inner, .. }, to) if **inner == *to || inner.plain() == *to => {
                Ok(Typed::new(into.clone(), typed.value.kind))
            }
            (from, Type::New { inner, .. })
                if *from == **inner
                    || (inner.plain() == **inner && from.plain() == **inner && !matches!(from, Type::New { .. })) =>
            {
                Ok(Typed::new(into.clone(), typed.value.kind))
            }
            // trace: D328
            (Type::Integer { kind, .. }, Type::Str(StrSize::View | StrSize::Growing)) => {
                let most = kind.most_text_bytes();
                Err(self.error(Diagnostic::refused_fix(
                    keyword.span.to(written_type.span),
                    format!("as str[len.max: {most} bytes]"),
                    "`as` never allocates, so text from `as` goes into a bounded `str`, held inside the value (D327, D328)",
                )))
            }
            (Type::Integer { .. } | Type::Float { .. }, Type::Str(_)) if body.purity != super::Purity::Plain => {
                Err(self.error(Diagnostic::not_designed(
                    expression.span,
                    "whether a `fn.pure` may allocate, as `as` can",
                    "not decided yet (standard 12.5)",
                )))
            }
            (Type::Integer { .. }, Type::Str(_)) => Ok(Typed::new(into.clone(), Kind::TextOf(Box::new(typed.value)))),
            (Type::Float { kind, .. }, Type::Str(_)) if *kind != super::types::FloatKind::Fast => {
                Ok(Typed::new(into.clone(), Kind::TextOf(Box::new(typed.value))))
            }
            (
                Type::Float { bits: from_bits, kind: super::types::FloatKind::Ieee, .. },
                Type::Float { bits, kind, bounds: None, .. },
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
        if body.changing.iter().any(|each| *each == name.text) {
            return Err(self.error(Diagnostic::refused(
                name.span,
                format!("a `for` is changing `{}` through its items here", name.text),
                "`Lend.mut` means nothing else uses it meanwhile, and the `for` is using it (D2, D180)",
            )));
        }
        if body.lent_receiver.as_deref() == Some(name.text.as_str()) {
            return Err(self.error(Diagnostic::not_designed(
                name.span,
                format!("reading `{}` in a call that lends it with `Lend.mut`", name.text),
                NOT_DECIDED,
            )));
        }
        let Some(index) = body.find(&name.text) else {
            if let Some((value_type, value)) = self.constants.get(&name.text).cloned() {
                let mut typed = Typed::new(value_type.clone(), value.expression(value_type.machine()).kind);
                typed.variable = Some(name.text.clone());
                return Ok(typed);
            }
            return Err(self.error(Diagnostic::refused(
                name.span,
                format!("`{}` isn't declared", name.text),
                "`let` and `const` declare a variable with its type and value (D57, D83)",
            )));
        };
        let variable = body.variables[index].clone();
        if let Type::Failure { .. } = variable.value_type {
            return Err(self.error(Diagnostic::refused(
                name.span,
                format!("`{}` names a failure, which is for reading its data", name.text),
                "`case negative $f:` names the failure, so `$f.val` reads its data (D139)",
            )));
        }
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
        let mut typed = Typed::new(variable.value_type.read_as().clone(), Kind::Local(variable.local));
        typed.variable = Some(name.text.clone());
        if !variable.value_type.is_copied() && variable.origin != Origin::Constant {
            let site = body.site(name.span);
            body.events.push(crate::flow::Event::Read {
                place: crate::flow::Place { local: variable.local, path: Vec::new(), shown: String::new() },
                spot: crate::flow::Spot { variable: name.span, place: name.span, site, marker: site },
            });
        }
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
                let mark = body.events.len();
                let right_typed = self.condition_value(body, right);
                if !both {
                    body.maybe_since(mark);
                }
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
                if let Some(parameter) = [&left_typed.value_type, &right_typed.value_type]
                    .into_iter()
                    .find(|value_type| matches!(value_type, Type::Param { .. }))
                    .cloned()
                {
                    let written = self.slice(operator_span).to_string();
                    self.parameter_operator(expression.span, &written, &parameter)?;
                    // A generic's own body is only checked, so the comparison needs no code.
                    return Ok(Typed::new(Type::Bool, Kind::Zero));
                }
                if is_own_type(&left_typed.value_type) || is_own_type(&right_typed.value_type) {
                    let written = self.slice(operator_span).to_string();
                    return self.own_operator(expression, &written, (left, right), (left_typed, right_typed));
                }
                // A plain literal against a unit needs the unit (D100, D101).
                for (side, typed, other) in [(right, &right_typed, &left_typed), (left, &left_typed, &right_typed)] {
                    if typed.value_type.unit().is_none()
                        && (typed.literal.is_some() || typed.float_literal.is_some())
                        && same_number(&typed.value_type, &other.value_type)
                        && let Some(unit) = other.value_type.unit()
                    {
                        let shown = self.slice(side.span).to_string();
                        return Err(self.error(Diagnostic::refused_fix(
                            side.span,
                            format!("{shown} {unit}"),
                            "the compiler checks units, and a unit is required where the type asks for one (D100, D101)",
                        )));
                    }
                }
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
        if let Some(reported) = self.new_in_operation(span, left, right, true) {
            return Err(reported);
        }
        // The same units compare, and then it's the numbers that do (D100). Between different number types, it's the
        // type that's wrong (D7).
        if (left_type.unit().is_some() || right_type.unit().is_some()) && same_number(left_type, right_type) {
            if !self.same_unit(left_type.unit(), right_type.unit()) {
                return Err(self.error(Diagnostic::refused(
                    span,
                    format!("this compares `{left_type}` and `{right_type}`"),
                    "the same units can add and compare, and mixing them is refused (D100)",
                )));
            }
            let (mut left, mut right) = (left.clone(), right.clone());
            left.value_type = left.value_type.with_unit(None);
            right.value_type = right.value_type.with_unit(None);
            return self.comparison(span, operator, &left, &right);
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
            (Type::Str(_), Type::Str(_)) => {
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
        let (mut left_typed, mut right_typed) = self.operands(body, left, right, expected)?;
        if let Some(parameter) = [&left_typed.value_type, &right_typed.value_type]
            .into_iter()
            .find(|value_type| matches!(value_type, Type::Param { .. }))
            .cloned()
        {
            let written = self.slice(operator_span).to_string();
            self.parameter_operator(expression.span, &written, &parameter)?;
            return Ok(Typed::new(parameter, Kind::Zero));
        }
        if is_own_type(&left_typed.value_type) || is_own_type(&right_typed.value_type) {
            let written = self.slice(operator_span).to_string();
            return self.own_operator(expression, &written, (left, right), (left_typed, right_typed));
        }
        // Units are checked first, then the maths runs on the numbers, and the result gets its unit (D100, D242).
        let left_unit = left_typed.value_type.unit().cloned();
        let right_unit = right_typed.value_type.unit().cloned();
        if left_unit.is_none() && right_unit.is_none() {
            let typed = self.arithmetic_on(expression, operator, operator_span, (left_typed, right_typed))?;
            return self.saturated(expression, operator, typed, expected);
        }
        let unit = self.maths_unit(expression, operator, left, right, (&left_typed, &right_typed))?;
        left_typed.value_type = left_typed.value_type.with_unit(None);
        right_typed.value_type = right_typed.value_type.with_unit(None);
        let mut typed = self.arithmetic_on(expression, operator, operator_span, (left_typed, right_typed))?;
        typed.value_type = typed.value_type.with_unit(unit);
        self.saturated(expression, operator, typed, expected)
    }

    /// Saturating maths sticks at the bounds of the type its result goes into, so a narrower written bound clamps
    /// what the hardware gave (D294, D372), and a float sticks at its written bounds (D373).
    fn saturated(
        &mut self,
        expression: &syntax::Expression,
        operator: &BinaryOperator,
        mut typed: Typed,
        expected: Option<&Type>,
    ) -> Check<Typed> {
        if !matches!(
            operator,
            BinaryOperator::SaturatingAdd | BinaryOperator::SaturatingSubtract | BinaryOperator::SaturatingMultiply
        ) {
            return Ok(typed);
        }
        // trace: D294, D372, D373
        match (&typed.value_type, expected) {
            (Type::Integer { kind, .. }, Some(Type::Integer { kind: wanted, bounds: Some(bounds), .. }))
                if kind == wanted =>
            {
                let machine_type = typed.value.machine_type.clone();
                let value = std::mem::replace(
                    &mut typed.value,
                    checked::Expression { kind: Kind::Zero, machine_type: machine_type.clone() },
                );
                typed.value = checked::Expression {
                    kind: Kind::Clamp { value: Box::new(value), min: bounds.min, max: bounds.max },
                    machine_type,
                };
                Ok(typed)
            }
            (Type::Integer { .. }, _) => Ok(typed),
            (Type::Float { .. }, Some(Type::Float { bounds: Some(bounds), .. })) => {
                let machine_type = typed.value.machine_type.clone();
                let value = std::mem::replace(
                    &mut typed.value,
                    checked::Expression { kind: Kind::Zero, machine_type: machine_type.clone() },
                );
                typed.value = checked::Expression {
                    kind: Kind::ClampFloat {
                        value: Box::new(value),
                        min: f64::from_bits(bounds.min),
                        max: f64::from_bits(bounds.max),
                    },
                    machine_type,
                };
                Ok(typed)
            }
            (Type::Float { .. }, _) => {
                let shown = expected.unwrap_or(&typed.value_type).to_string();
                Err(self.error(Diagnostic::not_designed(
                    expression.span,
                    format!("saturating `{shown}`, a float with no written bounds"),
                    "not decided yet, and the standard doesn't say",
                )))
            }
            _ => Ok(typed),
        }
    }

    /// The unit maths on units gives: `+` and `-` keep one unit, and `x` and `div` combine them (D100, D242).
    fn maths_unit(
        &mut self,
        expression: &syntax::Expression,
        operator: &BinaryOperator,
        left: &syntax::Expression,
        right: &syntax::Expression,
        (left_typed, right_typed): (&Typed, &Typed),
    ) -> Check<Option<super::types::Unit>> {
        use super::types::Unit;
        let (left_unit, right_unit) = (left_typed.value_type.unit(), right_typed.value_type.unit());
        let literal = |typed: &Typed| typed.literal.is_some() || typed.float_literal.is_some();
        match operator {
            BinaryOperator::Add | BinaryOperator::Subtract
                if left_unit.and_then(super::units::unit_mark).is_some()
                    || right_unit.and_then(super::units::unit_mark).is_some() =>
            {
                self.point_maths(expression, operator, left, right, (left_typed, right_typed))
            }
            BinaryOperator::Add | BinaryOperator::Subtract => {
                if self.same_unit(left_unit, right_unit) {
                    return Ok(left_unit.cloned());
                }
                for (side, typed, other) in [(right, right_typed, left_unit), (left, left_typed, right_unit)] {
                    if typed.value_type.unit().is_none()
                        && literal(typed)
                        && let Some(other) = other
                    {
                        let shown = self.slice(side.span).to_string();
                        return Err(self.error(Diagnostic::refused_fix(
                            side.span,
                            format!("{shown} {other}"),
                            "the compiler checks units, and a unit is required where the type asks for one (D100, D101)",
                        )));
                    }
                }
                Err(self.error(Diagnostic::refused(
                    expression.span,
                    format!("this mixes `{}` and `{}`", left_typed.value_type, right_typed.value_type),
                    "the same units can add and compare, and mixing them is refused (D100)",
                )))
            }
            BinaryOperator::Multiply | BinaryOperator::Divide(_) => {
                if left_unit.is_none() || right_unit.is_none() {
                    return Err(self.error(Diagnostic::not_designed(
                        expression.span,
                        "scaling a unit by a plain number, and what unit comes back",
                        NOT_DECIDED,
                    )));
                }
                let sign = if matches!(operator, BinaryOperator::Multiply) { 1 } else { -1 };
                Ok(Unit::combine(left_unit, right_unit, sign))
            }
            _ => Err(self.not_yet(expression.span, "this maths on numbers with units".to_string())),
        }
    }

    /// An operator on a struct or union, which calls the `op` it defines for those types. `above`, `at_least`,
    /// `at_most` and `!=` follow from `below` and `==` (D236, D237, D278).
    fn own_operator(
        &mut self,
        expression: &syntax::Expression,
        written: &str,
        (left, right): (&syntax::Expression, &syntax::Expression),
        (left_typed, right_typed): (Typed, Typed),
    ) -> Check<Typed> {
        const WHY: &str = "a struct or union gets an operator by defining it with `op`, and it applies only where its values' types match (D236, D237)";
        let (named, swap, negate) = match written {
            "above" => ("below", true, false),
            "at_least" => ("below", false, true),
            "at_most" => ("below", true, true),
            "!=" => ("==", false, true),
            other => (other, false, false),
        };
        let mut sides = [(left, left_typed), (right, right_typed)];
        if swap {
            sides.swap(0, 1);
        }
        let mut key = (named.to_string(), sides[0].1.value_type.to_string(), sides[1].1.value_type.to_string());
        // A literal takes the number type the operator asks for there, as `$total div.floor 2` does.
        if !self.operators.contains_key(&key) {
            for index in 0..2 {
                let literal = sides[index].1.literal.is_some() || sides[index].1.float_literal.is_some();
                if !literal {
                    continue;
                }
                let other = sides[1 - index].1.value_type.to_string();
                let found = self
                    .operators
                    .keys()
                    .find(|(op, a, b)| op == named && if index == 0 { *b == other } else { *a == other });
                if let Some(found) = found.cloned()
                    && let Some(signature) =
                        self.operators.get(&found).and_then(|name| self.functions.get(name)).cloned()
                    && let Some((_, wanted, _)) = signature.parameters.get(index)
                {
                    // A literal reads no variables, so it's typed again on its own.
                    let retyped = self.expression(&mut Body::default(), sides[index].0, Some(wanted))?;
                    let value = self.convert_into(sides[index].0.span, retyped, wanted)?;
                    sides[index].1 = Typed::new(wanted.clone(), value.kind);
                    key = found;
                    break;
                }
            }
        }
        let Some(signature) = self.operators.get(&key).and_then(|name| self.functions.get(name)).cloned() else {
            let what = if named == written {
                format!("there's no `op {named}` for `{}` and `{}`", key.1, key.2)
            } else {
                format!("there's no `op {named}` for `{}` and `{}`, which `{written}` follows from", key.1, key.2)
            };
            return Err(self.error(Diagnostic::refused(expression.span, what, WHY)));
        };
        let [(first, first_typed), (second, second_typed)] = sides;
        let mut arguments = Vec::new();
        for ((side, typed), (_, parameter_type, _)) in
            [(first, first_typed), (second, second_typed)].into_iter().zip(&signature.parameters)
        {
            let _ = side;
            // A lent value goes in a temporary whose address is passed, which reads the same as lending it in place.
            arguments.push(if parameter_type.is_copied() {
                checked::Argument::Value(typed.value)
            } else {
                checked::Argument::Temporary(typed.value)
            });
        }
        let call = Typed::new(
            signature.returns.clone(),
            Kind::Call { function: signature.id, arguments, span: expression.span },
        );
        if negate {
            return Ok(Typed::new(Type::Bool, Kind::Not(Box::new(call.value))));
        }
        Ok(call)
    }

    /// `+` and `-` on points and spans: a point plus a span is a point, a point minus a point is a span, and spans add
    /// (D249, D251).
    fn point_maths(
        &mut self,
        expression: &syntax::Expression,
        operator: &BinaryOperator,
        left: &syntax::Expression,
        right: &syntax::Expression,
        (left_typed, right_typed): (&Typed, &Typed),
    ) -> Check<Option<super::types::Unit>> {
        use super::units::{unit_mark, unmarked};
        let (left_unit, right_unit) = (left_typed.value_type.unit(), right_typed.value_type.unit());
        for (side, typed) in [(left, left_typed), (right, right_typed)] {
            let bare = typed.value_type.unit().is_some_and(|unit| unit_mark(unit).is_none());
            if bare && (typed.literal.is_some() || typed.float_literal.is_some()) {
                let shown = self.slice(side.span).to_string();
                return Err(self.error(Diagnostic::not_designed(
                    side.span,
                    format!("whether `{shown}` in maths is a point or a span"),
                    "not decided yet (standard 12.2)",
                )));
            }
        }
        let (Some(base), Some(other)) = (left_unit.and_then(unmarked), right_unit.and_then(unmarked)) else {
            return Err(self.error(Diagnostic::refused(
                expression.span,
                format!("this mixes `{}` and `{}`", left_typed.value_type, right_typed.value_type),
                "the same units can add and compare, and mixing them is refused (D100)",
            )));
        };
        if base != other {
            return Err(self.error(Diagnostic::refused(
                expression.span,
                format!("this mixes `{}` and `{}`", left_typed.value_type, right_typed.value_type),
                "the same units can add and compare, and mixing them is refused (D100)",
            )));
        }
        let adding = matches!(operator, BinaryOperator::Add);
        let mark = match (left_unit.and_then(unit_mark), right_unit.and_then(unit_mark), adding) {
            (Some("point"), Some("span"), _) | (Some("span"), Some("point"), true) => "point",
            (Some("span"), Some("span"), _) | (Some("point"), Some("point"), false) => "span",
            (Some("point"), Some("point"), true) => {
                return Err(self.error(Diagnostic::refused(
                    expression.span,
                    "this adds a point to a point",
                    "a point plus a span is a point, a point minus a point is a span, and spans add, while a point plus a point is refused (D249, D251)",
                )));
            }
            _ => {
                return Err(self.error(Diagnostic::not_designed(
                    expression.span,
                    "taking a point from a span",
                    NOT_DECIDED,
                )));
            }
        };
        Ok(Some(super::types::Unit::named(&format!("{base}.{mark}"))))
    }

    /// A number with its unit: `3600 time.seconds` (D99, D101).
    fn with_unit(
        &mut self,
        body: &mut Body,
        expression: &syntax::Expression,
        value: &syntax::Expression,
        unit: &syntax::Path,
        expected: Option<&Type>,
    ) -> Check<Typed> {
        let unit = self.resolve_unit(unit, true)?;
        if !Self::is_literal_number(value) {
            return Err(self.not_yet(expression.span, "a unit after something other than a number".to_string()));
        }
        let hint = expected.map(|hint| hint.with_unit(None));
        let mut typed = self.expression(body, value, hint.as_ref())?;
        typed.value_type = typed.value_type.with_unit(Some(unit));
        Ok(typed)
    }

    #[allow(clippy::too_many_lines)]
    fn arithmetic_on(
        &mut self,
        expression: &syntax::Expression,
        operator: &BinaryOperator,
        operator_span: Span,
        (left_typed, right_typed): (Typed, Typed),
    ) -> Check<Typed> {
        let operator_name = self.slice(operator_span).to_string();
        let (left_type, right_type) = (left_typed.value_type.clone(), right_typed.value_type.clone());
        if left_type.is_unknown() || right_type.is_unknown() {
            return Err(Reported);
        }
        if let Some(reported) = self.new_in_operation(expression.span, &left_typed, &right_typed, false) {
            return Err(reported);
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
            BinaryOperator::WrappingSubtract => (Arithmetic::WrappingSubtract, None, Vec::new()),
            BinaryOperator::WrappingMultiply => (Arithmetic::WrappingMultiply, None, Vec::new()),
            BinaryOperator::SaturatingAdd => (Arithmetic::SaturatingAdd, None, Vec::new()),
            BinaryOperator::SaturatingSubtract => (Arithmetic::SaturatingSubtract, None, Vec::new()),
            BinaryOperator::SaturatingMultiply => (Arithmetic::SaturatingMultiply, None, Vec::new()),
            // trace: D404
            BinaryOperator::RotateLeft => (Arithmetic::RotateLeft, None, Vec::new()),
            BinaryOperator::RotateRight => (Arithmetic::RotateRight, None, Vec::new()),
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
                Some(Diagnostic::not_designed(
                    expression.span,
                    format!("what happens when `{operator_name}` overflows"),
                    NOT_DECIDED,
                )),
                pair(),
            ),
            BinaryOperator::ShiftRight => (
                Arithmetic::ShiftRight,
                Some(Diagnostic::not_designed(
                    expression.span,
                    format!("whether `{operator_name}` on a negative number keeps its sign"),
                    NOT_DECIDED,
                )),
                vec![When::Text("`".into()), When::Operand(0), When::Text("`".into())],
            ),
            _ => return Err(self.not_yet(operator_span, format!("the operator `{operator_name}`"))),
        };
        let first = failure_diagnostic.map(|diagnostic| Box::new(failure(self, diagnostic, &when)));
        let mut more = Vec::new();
        if matches!(
            arithmetic,
            Arithmetic::ShiftLeft | Arithmetic::ShiftRight | Arithmetic::RotateLeft | Arithmetic::RotateRight
        ) {
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
        let saturating = matches!(
            operator,
            BinaryOperator::SaturatingAdd | BinaryOperator::SaturatingSubtract | BinaryOperator::SaturatingMultiply
        );
        let arithmetic = match operator {
            BinaryOperator::Add | BinaryOperator::SaturatingAdd => checked::FloatArithmetic::Add,
            BinaryOperator::Subtract | BinaryOperator::SaturatingSubtract => checked::FloatArithmetic::Subtract,
            BinaryOperator::Multiply | BinaryOperator::SaturatingMultiply => checked::FloatArithmetic::Multiply,
            BinaryOperator::WrappingAdd | BinaryOperator::WrappingSubtract | BinaryOperator::WrappingMultiply => {
                // trace: D373
                let name = self.slice(operator_span).to_string();
                return Err(self.error(Diagnostic::refused(
                    expression.span,
                    format!("`{name}` is for integers"),
                    "floats don't wrap: too big becomes infinity, so wrapping means nothing for them (D295, D373)",
                )));
            }
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
            // Saturating sticks infinity at a bound instead (D373).
            let infinity = if saturating { None } else { Some(Box::new(infinity)) };
            let nan = failure(
                self,
                Diagnostic::new(DiagnosticKind::CheckFailed, expression.span, "this makes NaN", why),
                &when,
            );
            (infinity, Some(Box::new(nan)))
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
                // Past the first comparison, an operand only runs if the ones before held.
                let mark = body.events.len();
                let result = self.expression(body, operand, None);
                if index >= 2 {
                    body.maybe_since(mark);
                }
                match result {
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
        // A chain over structs calls their `op below` and `op ==`, one pair at a time (D236).
        if typed.iter().any(|each| is_own_type(&each.value_type)) {
            let mut joined: Option<checked::Expression> = None;
            for (index, (_, operator_span)) in operators.iter().enumerate() {
                let written = self.slice(*operator_span).to_string();
                let span = operands[index].span.to(operands[index + 1].span);
                let pair = syntax::Expression { kind: ExpressionKind::Group(Box::new(operands[index].clone())), span };
                let compared = self.own_operator(
                    &pair,
                    &written,
                    (&operands[index], &operands[index + 1]),
                    (typed[index].clone(), typed[index + 1].clone()),
                )?;
                joined = Some(match joined {
                    None => compared.value,
                    Some(before) => checked::Expression {
                        kind: Kind::And(Box::new(before), Box::new(compared.value)),
                        machine_type: MachineType::Bool,
                    },
                });
            }
            let Some(joined) = joined else { return Err(Reported) };
            return Ok(Typed::new(Type::Bool, joined.kind));
        }
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

    /// A method call, `$r.%%warm_up(by: 2)`: the receiver is lent with `%`, or to change with `%%` (D130, D211).
    pub fn method_call(
        &mut self,
        body: &mut Body,
        expression: &syntax::Expression,
        receiver: &syntax::Expression,
        method: &syntax::Path,
        arguments: &[syntax::Argument],
    ) -> Check<Typed> {
        body.calls.push(expression.span);
        let result = self.method_inside(body, expression, receiver, method, arguments);
        body.calls.pop();
        result
    }

    fn method_inside(
        &mut self,
        body: &mut Body,
        expression: &syntax::Expression,
        receiver: &syntax::Expression,
        method: &syntax::Path,
        arguments: &[syntax::Argument],
    ) -> Check<Typed> {
        let mut written = method.text();
        let receiver_type = self.expression_type_quietly(body, receiver);
        let on_list = matches!(receiver_type, Some(Type::List { .. }));
        // A kit method's short name stands for its path on the type that path names (D178).
        if on_list
            && let Some(full) = self.method_uses.get(&written)
            && let Some(method_name) = full.strip_prefix("std::coll::list.")
        {
            written = format!("std::{method_name}");
        } else if on_list && !written.contains("::") && written.trim_start_matches('%') == "append" {
            // A method from a kit says its path, because bare means base or your own (D115).
            let receiver_text = self.slice(receiver.span).to_string();
            let rest =
                self.slice(Span::new(self.file, method.span.end as usize, expression.span.end as usize)).to_string();
            return Err(self.error(Diagnostic::refused_fix(
                expression.span,
                format!("{receiver_text}.std::%%append{rest}"),
                "a method from a kit says its path, because bare means base or your own (D115)",
            )));
        }
        if written.starts_with("std::") {
            let name_span = method.last().map_or(method.span, |segment| segment.name.span);
            if !matches!(written.as_str(), "std::%%append") {
                return Err(self.error(Diagnostic::not_designed(
                    name_span,
                    format!("the method `{written}`, since `std`'s full contents aren't chosen"),
                    NOT_DECIDED,
                )));
            }
        }
        if written.contains("::") {
            return Err(self.not_yet(method.span, format!("the method `{written}`")));
        }
        let receiver_typed = self.expression(body, receiver, None)?;
        let owner = match &receiver_typed.value_type {
            Type::Struct { name, .. } | Type::Union { name, .. } | Type::New { name, .. } => name.clone(),
            Type::Unknown => return Err(Reported),
            other @ (Type::List { .. } | Type::Array { .. }) => {
                return Err(self.error(Diagnostic::not_designed(
                    expression.span,
                    format!("a method of your own on `{other}`"),
                    NOT_DECIDED,
                )));
            }
            other => return Err(self.not_yet(method.span, format!("a method on `{other}`"))),
        };
        let bare = written.trim_start_matches('%');
        let Some((full, signature)) = [format!("{owner}.%{bare}"), format!("{owner}.%%{bare}")]
            .into_iter()
            .find_map(|name| self.functions.get(&name).cloned().map(|signature| (name, signature)))
        else {
            return Err(self.error(Diagnostic::refused(
                method.span,
                format!("`{owner}` has no method `{written}`"),
                "a method is declared with its type in the name, as in `reading.%warm_up` (D130)",
            )));
        };
        let changes = full.contains(".%%");
        if changes != written.starts_with("%%") {
            let fixed = if changes { format!("%%{bare}") } else { format!("%{bare}") };
            return Err(self.error(Diagnostic::refused_fix(
                method.span,
                fixed,
                "`.%` lends a receiver read-only and `.%%` lends it to change, so the call shows which (D211)",
            )));
        }
        // The receiver, then the rest, matched by the method's labels without its receiver.
        let receiver_argument = if changes && is_item(receiver) {
            let Some(root) = root_name(receiver) else { return Err(Reported) };
            let Some(variable) = body.lookup(&root.text).cloned() else { return Err(Reported) };
            if !variable.mutable {
                return Err(self.error(Diagnostic::refused(
                    receiver.span,
                    format!("`{}` can't change, so neither can its item", root.text),
                    "a field or item can be changed if the whole variable is `let mut`, or the parameter `Lent.mut` (D125, D133)",
                )));
            }
            let Some(reached) = self.reach(body, receiver)? else { return Err(Reported) };
            checked::Argument::Place(checked::Place { local: variable.local, steps: reached.steps })
        } else if changes {
            let ExpressionKind::Variable(name) = &receiver.kind else {
                if let Some((local, path, root)) = self.field_place(body, receiver) {
                    let Some(variable) = body.lookup(&root).cloned() else { return Err(Reported) };
                    if !variable.mutable {
                        return Err(self.error(Diagnostic::refused(
                            receiver.span,
                            format!("`{root}` can't change, so neither can its field"),
                            "a field or item can be changed if the whole variable is `let mut`, or the parameter `Lent.mut` (D125, D133)",
                        )));
                    }
                    return self.finish_method(
                        body,
                        expression,
                        &signature_for(self, &owner, bare),
                        checked::Argument::FieldAddress { local, path },
                        arguments,
                    );
                }
                return Err(self.error(Diagnostic::not_designed(
                    receiver.span,
                    "lending a fresh value with `Lend.mut`",
                    NOT_DECIDED,
                )));
            };
            let Some(variable) = body.lookup(&name.text).cloned() else { return Err(Reported) };
            // Lending all of it to change changes whatever part a `for` is reading (D2, D128).
            if body.reading.iter().chain(&body.reading_parts).any(|each| *each == name.text) {
                return Err(self.error(Diagnostic::refused(
                    receiver.span,
                    format!("a `for` is reading `{}` in place here", name.text),
                    "`Lend.mut` means nothing else uses it meanwhile, and `for` is using it (D2)",
                )));
            }
            if !variable.mutable {
                return Err(self.error(Diagnostic::refused(
                    receiver.span,
                    format!("`{}` can't change, so it can't be lent with `Lend.mut`", name.text),
                    "`let` declares a variable that can't change, and `let mut` one that can (D3)",
                )));
            }
            body.take_read(variable.local);
            let site = body.site(name.span);
            body.events.push(crate::flow::Event::LendMut {
                place: crate::flow::Place { local: variable.local, path: Vec::new(), shown: name.text.clone() },
                spot: crate::flow::Spot { variable: name.span, place: name.span, site, marker: site },
            });
            checked::Argument::Address(variable.local)
        } else if receiver_typed.value_type.is_copied() {
            // A lent number is passed as its value, as its parameter takes it.
            checked::Argument::Value(receiver_typed.value)
        } else {
            match place_of(body, receiver) {
                Some((local, path)) if path.is_empty() => checked::Argument::Address(local),
                _ => checked::Argument::Temporary(receiver_typed.value),
            }
        };
        self.finish_method(body, expression, &(full, signature), receiver_argument, arguments)
    }

    /// The rest of a method call, once its receiver's argument is known.
    fn finish_method(
        &mut self,
        body: &mut Body,
        expression: &syntax::Expression,
        found: &(String, super::Signature),
        receiver_argument: checked::Argument,
        arguments: &[syntax::Argument],
    ) -> Check<Typed> {
        let (full, signature) = (found.0.clone(), found.1.clone());
        if body.in_contract && signature.purity == super::Purity::Plain {
            return Err(self.error(Diagnostic::refused(
                expression.span,
                format!("`{full}` isn't pure, so a contract can't call it"),
                "contracts can only call pure functions (D86)",
            )));
        }
        // Reading a receiver lent with `Lend.mut` in its own call's arguments isn't decided (standard 12.6).
        let lent_receiver = match (&receiver_argument, &expression.kind) {
            (checked::Argument::Address(_), ExpressionKind::MethodCall { receiver, .. }) if full.contains(".%%") => {
                root_name(receiver).map(|name| name.text.clone())
            }
            _ => None,
        };
        let outer = std::mem::replace(&mut body.lent_receiver, lent_receiver);
        let result = self.finish_method_inside(body, expression, &full, &signature, receiver_argument, arguments);
        body.lent_receiver = outer;
        result
    }

    fn finish_method_inside(
        &mut self,
        body: &mut Body,
        expression: &syntax::Expression,
        full: &str,
        signature: &super::Signature,
        receiver_argument: checked::Argument,
        arguments: &[syntax::Argument],
    ) -> Check<Typed> {
        let full = full.to_string();
        let signature = signature.clone();
        let mut rest = signature.clone();
        rest.parameters.remove(0);
        rest.top.labels.remove(0);
        if !rest.top.label_units.is_empty() {
            rest.top.label_units.remove(0);
        }
        rest.top.defaults.remove(0);
        rest.name = full.clone();
        if arguments.len() > rest.parameters.len() {
            let extra = &arguments[rest.parameters.len()];
            return Err(self.error(Diagnostic::refused(
                extra.span,
                format!("this is one argument too many for `{full}`"),
                "a declared label must be written, and a parameter with no label is called without one (D113, D114)",
            )));
        }
        let matched = self.match_arguments(expression.span, &rest, arguments)?;
        let mut values = vec![receiver_argument];
        let mut changed = Vec::new();
        for (index, (each, (parameter_name, parameter_type, ownership))) in
            matched.iter().zip(&rest.parameters).enumerate()
        {
            let carried = rest.top.label_units.get(index).cloned().flatten();
            let outer = std::mem::replace(&mut self.label_unit, carried);
            let value = match each {
                super::contracts::Matched::Argument(argument) => {
                    self.argument(body, argument, parameter_name, parameter_type, *ownership, &mut changed)
                }
                super::contracts::Matched::Default(default) => {
                    self.value_into(body, default, parameter_type).map(checked::Argument::Value)
                }
            };
            self.label_unit = outer;
            values.push(value?);
        }
        if super::contracts::needs_site(&signature.top) {
            let site = body.sites.len();
            body.sites.push(self.site(expression.span, "called here"));
            values.push(checked::Argument::Value(checked::Expression {
                kind: Kind::Site(site),
                machine_type: MachineType::Address,
            }));
        }
        let mut typed = Typed::new(
            signature.returns.clone(),
            Kind::Call { function: signature.id, arguments: values, span: expression.span },
        );
        if let Some(layout) = signature.result {
            typed.value.machine_type = super::failures::result_machine_type(layout);
            return Ok(self.crash_on_failure(expression.span, &signature, typed.value));
        }
        Ok(typed)
    }

    /// A field inside a variable, as its local, the fields down to it, and the variable's name.
    pub fn field_place(
        &self,
        body: &Body,
        expression: &syntax::Expression,
    ) -> Option<(checked::LocalId, Vec<usize>, String)> {
        let mut names = Vec::new();
        let mut at = expression;
        while let ExpressionKind::Field { base, field } = &at.kind {
            names.push(field.text.clone());
            at = base;
        }
        let ExpressionKind::Variable(root) = &at.kind else { return None };
        let variable = body.lookup(&root.text)?;
        let mut current = variable.value_type.clone();
        let mut path = Vec::new();
        for name in names.iter().rev() {
            let Type::Struct { id, .. } = &current else { return None };
            let info = &self.aggregates.structs[*id];
            let index = info.fields.iter().position(|field| field.name == *name)?;
            path.push(index);
            current = info.fields[index].value_type.clone();
        }
        Some((variable.local, path, root.text.clone()))
    }

    /// A call, before what happens when it fails is known.
    pub fn call_unhandled(
        &mut self,
        body: &mut Body,
        expression: &syntax::Expression,
        callee: &syntax::Expression,
        arguments: &[syntax::Argument],
    ) -> Check<Typed> {
        body.calls.push(expression.span);
        let result = self.call_inside(body, expression, callee, arguments);
        body.calls.pop();
        result
    }

    fn call_inside(
        &mut self,
        body: &mut Body,
        expression: &syntax::Expression,
        callee: &syntax::Expression,
        arguments: &[syntax::Argument],
    ) -> Check<Typed> {
        let expected = self.call_expected.take();
        // A list's kit method called with a plain `.`, which never runs code (D8).
        if let ExpressionKind::Field { base, field } = &callee.kind
            && matches!(self.expression_type_quietly(body, base), Some(Type::List { .. }))
            && (field.text == "append"
                || ["%", "%%"].iter().any(|mark| {
                    self.method_uses
                        .get(&format!("{mark}{}", field.text))
                        .is_some_and(|full| full.starts_with("std::coll::list."))
                }))
        {
            let base_text = self.slice(base.span).to_string();
            let rest =
                self.slice(Span::new(self.file, callee.span.end as usize, expression.span.end as usize)).to_string();
            return Err(self.error(Diagnostic::refused_fix(
                expression.span,
                format!("{base_text}.std::%%append{rest}"),
                "a `.` never runs code, and a `.%` always does (D8)",
            )));
        }
        if let ExpressionKind::Field { base, field } = &callee.kind
            && let Ok(typed) = self.expression(body, base, None)
            && let Type::Struct { name, .. } | Type::Union { name, .. } = &typed.value_type
            && (self.functions.contains_key(&format!("{name}.%{}", field.text))
                || self.functions.contains_key(&format!("{name}.%%{}", field.text)))
        {
            let mark = if self.functions.contains_key(&format!("{name}.%%{}", field.text)) { "%%" } else { "%" };
            let base_text = self.slice(base.span).to_string();
            let arguments_text =
                self.slice(Span::new(self.file, callee.span.end as usize, expression.span.end as usize)).to_string();
            return Err(self.error(Diagnostic::refused_fix(
                expression.span,
                format!("{base_text}.{mark}{}{arguments_text}", field.text),
                "a `.` never runs code, and a `.%` always does (D8)",
            )));
        }
        let ExpressionKind::Path(path) = &callee.kind else {
            return Err(self.not_yet(callee.span, "calling this".to_string()));
        };
        let name = self.resolve_path(path);
        if name.contains(".%") && self.functions.contains_key(&name) {
            return Err(self.error(Diagnostic::not_designed(
                expression.span,
                "calling a method by its full name, as a plain function",
                NOT_DECIDED,
            )));
        }
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
        if let Some(signature) = self.functions.get(&name).cloned() {
            if body.in_contract && signature.purity == super::Purity::Plain {
                return Err(self.error(Diagnostic::refused(
                    callee.span,
                    format!("`{name}` isn't pure, so a contract can't call it"),
                    "contracts can only call pure functions (D86)",
                )));
            }
            if body.purity == super::Purity::Baked && signature.purity != super::Purity::Baked {
                return Err(self.error(Diagnostic::refused(
                    callee.span,
                    format!("`{name}` isn't `fn.pure.baked`, so a baked function can't call it"),
                    "baked code can call normal functions marked `fn.pure.baked`, and only those (D91)",
                )));
            }
            if body.purity == super::Purity::Pure && signature.purity == super::Purity::Plain {
                return Err(self.error(Diagnostic::refused(
                    callee.span,
                    format!("`{name}` isn't pure, so a pure function can't call it"),
                    "a pure function refuses calling a function that isn't pure (D86)",
                )));
            }
        }
        if super::clashes(&name) || self.functions.contains_key(&format!("my::{name}")) {
            let shown = self.slice(expression.span).to_string();
            return Err(self.error(Diagnostic::refused_fix(
                expression.span,
                format!("my::{shown}"),
                "a name of yours that clashes is written with `my::` (D75)",
            )));
        }
        let Some(mut signature) = self.functions.get(&name).cloned() else {
            return Err(self.error(Diagnostic::refused(
                callee.span,
                format!("the function `{name}` isn't declared"),
                "a function is declared with `fn` and `[ ]` (D23)",
            )));
        };
        if self.generics.functions.contains_key(&name) {
            signature = self.generic_signature(body, expression.span, &name, arguments, expected)?;
        }
        if let Type::Borrow { .. } = signature.returns
            && !std::mem::replace(&mut body.borrow_call, false)
        {
            return Err(self.error(Diagnostic::not_designed(
                expression.span,
                "using a returned borrow anywhere but in a stored borrow or a `return`",
                NOT_DECIDED,
            )));
        }
        let matched = self.match_arguments(expression.span, &signature, arguments)?;
        let mut values = Vec::new();
        let mut failed = false;
        let mut changed: Vec<Handed> = Vec::new();
        for (index, (each, (parameter_name, parameter_type, ownership))) in
            matched.iter().zip(&signature.parameters).enumerate()
        {
            let outer =
                std::mem::replace(&mut self.label_unit, signature.top.label_units.get(index).cloned().flatten());
            let result = match each {
                super::contracts::Matched::Argument(argument) => {
                    self.argument(body, argument, parameter_name, parameter_type, *ownership, &mut changed)
                }
                super::contracts::Matched::Default(default) => {
                    self.value_into(body, default, parameter_type).map(checked::Argument::Value)
                }
            };
            self.label_unit = outer;
            match result {
                Ok(value) => values.push(value),
                Err(Reported) => failed = true,
            }
        }
        if failed {
            return Err(Reported);
        }
        if super::contracts::needs_site(&signature.top) {
            let site = body.sites.len();
            body.sites.push(self.site(expression.span, "called here"));
            values.push(checked::Argument::Value(checked::Expression {
                kind: Kind::Site(site),
                machine_type: MachineType::Address,
            }));
        }
        if let Type::Borrow { sources, .. } = &signature.returns {
            body.call_borrows = Some(self.borrowed_arguments(body, &signature, &matched, sources)?);
        }
        let mut typed = Typed::new(
            signature.returns.clone(),
            Kind::Call { function: signature.id, arguments: values, span: expression.span },
        );
        if let Some(layout) = signature.result {
            typed.value.machine_type = super::failures::result_machine_type(layout);
        }
        if let Type::Borrow { .. } = signature.returns {
            typed.value.machine_type = MachineType::Address;
        }
        Ok(typed)
    }

    /// At a call that hands back a borrow, its sources become the arguments, which have to be named values (D170).
    /// Through a stored borrow, they're what it borrows (D174).
    fn borrowed_arguments(
        &mut self,
        body: &Body,
        signature: &super::Signature,
        matched: &[super::contracts::Matched],
        sources: &[String],
    ) -> Check<(Vec<String>, Vec<checked::LocalId>)> {
        let mut names = Vec::new();
        let mut locals = Vec::new();
        for source in sources {
            let Some(index) = signature.parameters.iter().position(|(name, _, _)| name == source) else { continue };
            let Some(super::contracts::Matched::Argument(argument)) = matched.get(index) else { continue };
            let Some(root) = root_name(&argument.value).filter(|_| place_of(body, &argument.value).is_some()) else {
                return Err(self.error(Diagnostic::refused(
                    argument.value.span,
                    "the result borrows from this, so it has to be a named value",
                    "a returned borrow's sources become the arguments, and a fresh one would end with the call (D170)",
                )));
            };
            let Some(variable) = body.lookup(&root.text) else { return Err(Reported) };
            match &variable.value_type {
                Type::Borrow { sources: theirs, .. } => {
                    for original in theirs {
                        if let Some(found) = body.lookup(original) {
                            locals.push(found.local);
                        }
                        names.push(original.clone());
                    }
                    locals.push(variable.local);
                }
                _ => {
                    names.push(root.text.clone());
                    locals.push(variable.local);
                }
            }
        }
        Ok((names, locals))
    }

    /// One argument, with the ownership word the parameter asks for (D2, D3, D184).
    #[allow(clippy::too_many_lines)]
    fn argument(
        &mut self,
        body: &mut Body,
        argument: &syntax::Argument,
        parameter_name: &str,
        parameter_type: &Type,
        ownership: syntax::ParameterOwnership,
        handed: &mut Vec<Handed>,
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
                if is_item(&argument.value) {
                    return self.lend_item(body, argument, parameter_type, handed);
                }
                if let Some(root) = root_name(&argument.value)
                    && body.lookup(&root.text).is_none()
                    && self.constants.contains_key(&root.text)
                {
                    return Err(self.error(Diagnostic::refused(
                        argument.value.span,
                        "a constant can't be lent with `Lend.mut`",
                        "a constant's value is fixed at compile time (D83)",
                    )));
                }
                let (Some((local, path)), Some(root)) =
                    (self.any_place(body, &argument.value), root_name(&argument.value))
                else {
                    return Err(self.not_yet(argument.value.span, "lending this with `Lend.mut`".to_string()));
                };
                self.hand_in_call(handed, &root.text, &path, argument.span, true)?;
                let typed = self.expression(body, &argument.value, Some(parameter_type))?;
                let Some(variable) = body.lookup(&root.text).cloned() else { return Err(Reported) };
                if !variable.mutable {
                    let what = if path.is_empty() {
                        format!("`{}` can't change, so it can't be lent with `Lend.mut`", root.text)
                    } else {
                        format!("`{}` can't change, so neither can its field", root.text)
                    };
                    return Err(self.error(Diagnostic::refused(
                        argument.value.span,
                        what,
                        "`let` declares a variable that can't change, and `let mut` one that can (D3)",
                    )));
                }
                if typed.value_type != *parameter_type && !typed.value_type.is_unknown() {
                    return Err(self.error(Diagnostic::refused(
                        argument.span,
                        format!("this is `{}`, where `{parameter_type}` is needed", typed.value_type),
                        CONVERT_WHY,
                    )));
                }
                body.take_read(variable.local);
                let site = body.site(root.span);
                body.events.push(crate::flow::Event::LendMut {
                    place: crate::flow::Place { local: variable.local, path: path.clone(), shown },
                    spot: crate::flow::Spot { variable: root.span, place: argument.value.span, site, marker: site },
                });
                if path.is_empty() {
                    Ok(checked::Argument::Address(local))
                } else {
                    Ok(checked::Argument::FieldAddress { local, path })
                }
            }
            _ if parameter_type.is_copied() => {
                // In a copy of a generic, the word was for a type parameter, and does nothing for a number (D244).
                if let Some((_, span)) = argument.ownership.filter(|_| !self.instance) {
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
                if let ExpressionKind::As { value: inner, keyword, written_type } = &argument.value.kind
                    && let Some((word, span)) = argument.ownership
                {
                    let typed = self.owned_as(body, word, span, &argument.value, inner, keyword, written_type)?;
                    return Ok(checked::Argument::Value(self.convert_into(
                        argument.value.span,
                        typed,
                        parameter_type,
                    )?));
                }
                if word.is_some() && !held {
                    return Err(self.error(Diagnostic::refused(
                        argument.value.span,
                        "a fresh value takes no word",
                        "a fresh value that has no holder yet takes no word (D2)",
                    )));
                }
                if let Some(root) = root_name(&argument.value)
                    && let Some((_, path)) = self.any_place(body, &argument.value)
                {
                    self.hand_in_call(handed, &root.text, &path, argument.span, word == Some(Ownership::Give))?;
                }
                let typed = self.expression(body, &argument.value, Some(parameter_type))?;
                if word == Some(Ownership::Give) && is_item(&argument.value) {
                    return Err(self.error(Diagnostic::not_designed(
                        argument.value.span,
                        "giving away one item of a collection",
                        NOT_DECIDED,
                    )));
                }
                if word == Some(Ownership::Give)
                    && let Some((local, path)) = self.any_place(body, &argument.value)
                    && let Some(root) = root_name(&argument.value)
                {
                    self.only_lent(body, local, root, &path, argument.value.span)?;
                    body.take_read(local);
                    let site = body.site(root.span);
                    body.events.push(crate::flow::Event::Give {
                        place: crate::flow::Place { local, path, shown },
                        spot: crate::flow::Spot { variable: root.span, place: argument.value.span, site, marker: site },
                    });
                }
                Ok(checked::Argument::Value(self.convert_into(argument.value.span, typed, parameter_type)?))
            }
            ParameterOwnership::Lent | ParameterOwnership::Bare => {
                // A view is only ever copied, so passing one on needs no word (D367, D368).
                let view = matches!(parameter_type, Type::Str(StrSize::View));
                match word {
                    Some(Ownership::Lend) => {}
                    None if held && view => {}
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
                    Some(_) => {
                        return Err(self.error(Diagnostic::refused_fix(
                            argument.span,
                            format!("Lend {shown}"),
                            "the call and the parameter both say it, and they must match (D2)",
                        )));
                    }
                }
                if let Some(root) = root_name(&argument.value) {
                    // An item counts as all of its list (D172).
                    let path = self.any_place(body, &argument.value).map(|(_, path)| path).unwrap_or_default();
                    self.hand_in_call(handed, &root.text, &path, argument.span, false)?;
                }
                let typed = self.expression(body, &argument.value, Some(parameter_type))?;
                let typed_type = typed.value_type.clone();
                if let (
                    Type::List { item: a, min: x, max: y, len_type: p },
                    Type::List { item: b, min: z, max: w, len_type: q },
                ) = (&typed_type, parameter_type)
                    && a == b
                    && x == z
                    && y == w
                    && p != q
                {
                    return Err(self.error(Diagnostic::not_designed(
                        argument.span,
                        format!("whether `{typed_type}` is accepted where `{parameter_type}` is asked for"),
                        NOT_DECIDED,
                    )));
                }
                // Lending any string gives a view of it (D327).
                let viewed = view && matches!(typed_type, Type::Str(_));
                if typed_type != *parameter_type && !typed_type.is_unknown() && !viewed {
                    return Err(self.error(Diagnostic::refused(
                        argument.value.span,
                        format!("this is `{typed_type}`, where `{parameter_type}` is needed"),
                        CONVERT_WHY,
                    )));
                }
                match self.any_place(body, &argument.value) {
                    Some((local, path)) if path.is_empty() => Ok(checked::Argument::Address(local)),
                    Some((local, path)) => Ok(checked::Argument::FieldAddress { local, path }),
                    None => Ok(checked::Argument::Temporary(typed.value)),
                }
            }
        }
    }

    /// `Lend.mut $a[offset.items: 0]`: an item lent to change, which counts as all of its list (D133, D172).
    fn lend_item(
        &mut self,
        body: &mut Body,
        argument: &syntax::Argument,
        parameter_type: &Type,
        handed: &mut Vec<Handed>,
    ) -> Check<checked::Argument> {
        let Some(root) = root_name(&argument.value) else { return Err(Reported) };
        self.hand_in_call(handed, &root.text, &[], argument.span, true)?;
        let Some(variable) = body.lookup(&root.text).cloned() else { return Err(Reported) };
        if !variable.mutable {
            return Err(self.error(Diagnostic::refused(
                argument.value.span,
                format!("`{}` can't change, so neither can its item", root.text),
                "a field or item can be changed if the whole variable is `let mut`, or the parameter `Lent.mut` (D125, D133)",
            )));
        }
        self.not_while_reading(body, &root.text, argument.value.span)?;
        let Some(reached) = self.reach(body, &argument.value)? else { return Err(Reported) };
        if reached.value_type != *parameter_type && !reached.value_type.is_unknown() {
            return Err(self.error(Diagnostic::refused(
                argument.span,
                format!("this is `{}`, where `{parameter_type}` is needed", reached.value_type),
                CONVERT_WHY,
            )));
        }
        let site = body.site(root.span);
        body.events.push(crate::flow::Event::LendMut {
            place: crate::flow::Place { local: variable.local, path: Vec::new(), shown: root.text.clone() },
            spot: crate::flow::Spot { variable: root.span, place: argument.value.span, site, marker: site },
        });
        Ok(checked::Argument::Place(checked::Place { local: variable.local, steps: reached.steps }))
    }

    /// Within one call, `Lend.mut` or `Give` of something means nothing else in the call can use it. Different fields
    /// are separate (D2, D173).
    fn hand_in_call(
        &mut self,
        handed: &mut Vec<Handed>,
        root: &str,
        path: &[usize],
        span: Span,
        exclusive: bool,
    ) -> Check<()> {
        let overlaps = |other: &[usize]| crate::flow::checks::overlaps(other, path);
        if let Some((_, _, earlier, _)) = handed.iter().find(|(name, other, _, other_exclusive)| {
            name == root && overlaps(other) && (exclusive || *other_exclusive)
        }) {
            return Err(self.error(
                Diagnostic::refused(
                    span,
                    format!("`{root}` is already handed off with `Lend.mut` in this call"),
                    "`Lend.mut` means nothing else uses it meanwhile, and `Give` means it's gone (D2)",
                )
                .with_because(*earlier, "that's here"),
            ));
        }
        handed.push((root.to_string(), path.to_vec(), span, exclusive));
        Ok(())
    }

    /// `as` on a number with a unit, or into one: taking it off, putting it on, or changing to another unit with the
    /// same base, by its exact ratio (D141, D255). `None` when units aren't involved.
    fn unit_conversion(&mut self, expression: &syntax::Expression, typed: &Typed, into: &Type) -> Option<Check<Typed>> {
        let from = &typed.value_type;
        if from.unit().is_none() && into.unit().is_none() {
            return None;
        }
        // Only between the same number type; another width isn't decided (standard 12.3).
        if from.with_unit(None).plain() != into.with_unit(None).plain() {
            return None;
        }
        let (from_unit, into_unit) = match (from.unit(), into.unit()) {
            (Some(a), Some(b)) => (a.clone(), b.clone()),
            // Taking a unit off, or putting one on, is the same number (D141).
            _ => return Some(Ok(Typed::new(into.clone(), typed.value.kind.clone()))),
        };
        let (Some(a), Some(b)) = (self.units.expand(&from_unit), self.units.expand(&into_unit)) else {
            return Some(Err(Reported));
        };
        let (from_mark, into_mark) = (super::units::unit_mark(&from_unit), super::units::unit_mark(&into_unit));
        if from_mark != into_mark {
            return Some(Err(self.error(Diagnostic::refused(
                expression.span,
                format!("`{from_unit}` and `{into_unit}` aren't both points or both spans"),
                "a point converts to a point and a span to a span, since a point applies offsets and a span only scales (D249, D255)",
            ))));
        }
        // A point moves its zero as it converts, and a span only scales (D250, D255).
        let (from_offset, into_offset) = if from_mark == Some("point") {
            let named = |unit: &super::types::Unit| unit.single().map_or(0.0, |name| self.units.offset(name));
            (named(&from_unit), named(&into_unit))
        } else {
            (0.0, 0.0)
        };
        let pi = a.pi - b.pi;
        if a.powers != b.powers {
            return Some(Err(self.error(Diagnostic::refused(
                expression.span,
                format!("`{from_unit}` and `{into_unit}` don't trace back to the same base"),
                "two units convert if they trace back to the same base, and otherwise they don't (D109)",
            ))));
        }
        // From one unit to the other, `from` is `ratio` times `into`.
        let Some(ratio) = super::units::Ratio::new(a.scale.top, a.scale.bottom)
            .and_then(|scale| scale.times(super::units::Ratio::new(b.scale.bottom, b.scale.top)?))
        else {
            return Some(Err(self.not_yet(expression.span, "this conversion".to_string())));
        };
        let machine = typed.value.machine_type.clone();
        let constant = |kind| checked::Expression { kind, machine_type: machine.clone() };
        let value = match from {
            Type::Integer { .. } if pi != 0 || from_offset != 0.0 || into_offset != 0.0 => {
                return Some(Err(self.not_yet(expression.span, "this conversion of a whole number".to_string())));
            }
            Type::Integer { .. } if ratio.bottom != 1 && self.narrowing.is_some() => {
                if ratio.top != 1 {
                    return Some(Err(self.error(Diagnostic::not_designed(
                        expression.span,
                        format!("an integer conversion whose ratio, {} per {}, isn't whole", ratio.top, ratio.bottom),
                        "not decided yet (standard 12.2)",
                    ))));
                }
                let word = self.narrowing.clone().unwrap_or_default();
                let (operator, more) = match word.as_str() {
                    "trunc" => (checked::Arithmetic::DivideTruncate, Vec::new()),
                    "floor" => (checked::Arithmetic::DivideFloor, Vec::new()),
                    _ => (
                        checked::Arithmetic::DivideExact,
                        vec![failure(
                            self,
                            Diagnostic::not_designed(
                                expression.span,
                                "what `div.exact` does when it isn't exact",
                                NOT_DECIDED_OPERATORS,
                            ),
                            &[
                                When::Text("`".into()),
                                When::Operand(0),
                                When::Text(" div.exact ".into()),
                                When::Operand(1),
                                When::Text("`".into()),
                            ],
                        )],
                    ),
                };
                constant(Kind::Arithmetic {
                    operator,
                    left: Box::new(typed.value.clone()),
                    right: Box::new(constant(Kind::Integer(ratio.bottom))),
                    failure: None,
                    more,
                })
            }
            Type::Integer { .. } => {
                if ratio.bottom != 1 {
                    let times = ratio.bottom / ratio.top.max(1);
                    return Some(Err(self.error(Diagnostic::refused(
                        expression.span,
                        format!("this goes to a unit {times} times bigger, which can lose something, so `as` says how it rounds"),
                        "a value changes unit with `as`: a float rounds to the nearest, an integer going to a smaller unit is exact, and one going to a bigger unit says how it rounds with `as.trunc`, `as.floor` or `as.exact` (D255)",
                    ))));
                }
                if ratio.top == 1 {
                    typed.value.clone()
                } else {
                    constant(Kind::Arithmetic {
                        operator: checked::Arithmetic::Multiply,
                        left: Box::new(typed.value.clone()),
                        right: Box::new(constant(Kind::Integer(ratio.top))),
                        failure: None,
                        more: Vec::new(),
                    })
                }
            }
            _ => {
                // A float rounds to the nearest, once for each step that does something.
                let mut value = typed.value.clone();
                let pi_steps = usize::try_from(pi.unsigned_abs()).unwrap_or(0);
                let pi_operator =
                    if pi > 0 { checked::FloatArithmetic::Multiply } else { checked::FloatArithmetic::Divide };
                #[allow(clippy::cast_precision_loss)]
                let steps: Vec<(checked::FloatArithmetic, f64)> = [
                    (checked::FloatArithmetic::Subtract, from_offset),
                    (checked::FloatArithmetic::Multiply, ratio.top as f64),
                    (checked::FloatArithmetic::Divide, ratio.bottom as f64),
                ]
                .into_iter()
                .chain(std::iter::repeat_n((pi_operator, std::f64::consts::PI), pi_steps))
                .chain([(checked::FloatArithmetic::Add, into_offset)])
                .collect();
                for (operator, by) in steps {
                    let does_nothing = match operator {
                        checked::FloatArithmetic::Add | checked::FloatArithmetic::Subtract => by == 0.0,
                        _ => (by - 1.0).abs() <= f64::EPSILON && !matches!(by, std::f64::consts::PI),
                    };
                    if !does_nothing {
                        value = constant(Kind::FloatArithmetic {
                            operator,
                            left: Box::new(value),
                            right: Box::new(constant(Kind::Float(by))),
                            infinity: None,
                            nan: None,
                        });
                    }
                }
                value
            }
        };
        Some(Ok(Typed::new(into.clone(), value.kind)))
    }

    /// Why a value's unit doesn't fit where it's going.
    fn unit_mismatch(&mut self, span: Span, typed: &Typed, into: &Type) -> Reported {
        let from = &typed.value_type;
        if from.unit().is_none()
            && (typed.literal.is_some() || typed.float_literal.is_some())
            && let Some(unit) = into.unit()
        {
            let shown = self.slice(span).to_string();
            return self.error(Diagnostic::refused_fix(
                span,
                format!("{shown} {unit}"),
                "a unit is required where the type asks for one (D101)",
            ));
        }
        if let (Some(from_unit), Some(into_unit)) = (from.unit(), into.unit())
            && let (Some(a), Some(b)) = (self.units.expand(from_unit), self.units.expand(into_unit))
            && a.powers == b.powers
            && self.is_compound(from_unit)
        {
            return self.error(Diagnostic::not_designed(
                span,
                format!("converting `{from_unit}` to `{into_unit}`, which a derived unit's conversion would do"),
                "not decided yet (standard 12.2)",
            ));
        }
        let why = if from.unit().is_some_and(|unit| self.is_compound(unit)) {
            "storing a value needs a declared unit whose powers match (D242)"
        } else {
            "the compiler checks units, and mixing them is refused (D100, D101)"
        };
        self.error(Diagnostic::refused(span, format!("this is `{from}`, where `{into}` is needed"), why))
    }

    /// The place an expression names: a variable, or a field inside one.
    pub fn any_place(&self, body: &Body, expression: &syntax::Expression) -> Option<(checked::LocalId, Vec<usize>)> {
        place_of(body, expression).or_else(|| self.field_place(body, expression).map(|(local, path, _)| (local, path)))
    }

    /// Refuses giving away what's only lent: a bare or `Lent.mut` parameter, or a stored borrow (D2).
    fn only_lent(
        &mut self,
        body: &Body,
        local: checked::LocalId,
        root: &syntax::Name,
        path: &[usize],
        place: Span,
    ) -> Check<()> {
        if body.lent.contains(&local) && !path.is_empty() {
            return Err(self.error(Diagnostic::refused(
                place,
                format!("`{}` is only borrowed here, so a field can't be moved out of it", root.text),
                "a field moves out only of something you own, a `Given` parameter or your own `let` (D185)",
            )));
        }
        if body.lent.contains(&local) {
            return Err(self.error(Diagnostic::refused(
                root.span,
                format!("`{}` is only lent here, so it can't be given away", root.text),
                "`Lend` is temporary access, and the caller keeps it (D2)",
            )));
        }
        Ok(())
    }

    /// `Give $r as wrapped`: `as` on a whole value says its ownership, and the word goes on the value (D149).
    #[allow(clippy::too_many_arguments)]
    fn owned_as(
        &mut self,
        body: &mut Body,
        ownership: syntax::Ownership,
        ownership_span: Span,
        whole: &syntax::Expression,
        inner: &syntax::Expression,
        keyword: &syntax::Path,
        written_type: &syntax::Type,
    ) -> Check<Typed> {
        // `Give $r as wrapped as rewrapped` gives `$r`, under every `as`.
        let mut inner = inner;
        let whole_inner = inner;
        while let ExpressionKind::As { value: deeper, .. } = &inner.kind {
            inner = deeper;
        }
        let marked = ownership_span.to(inner.span);
        let written = self.slice(inner.span).to_string();
        if matches!(ownership, syntax::Ownership::Lend | syntax::Ownership::LendMut) {
            return Err(self.error(Diagnostic::refused(
                marked,
                format!("`as` builds a new value, so `{}` doesn't fit here", ownership.word()),
                "`as` on a whole value says `Give` or `Copy` (D3, D149)",
            )));
        }
        let place = self.any_place(body, inner);
        if place.is_none() {
            return Err(self.error(Diagnostic::refused_fix(
                marked,
                written,
                "a fresh value that has no holder yet takes no word (D2, D149)",
            )));
        }
        body.handing = true;
        let typed = self.conversion(body, whole, whole_inner, keyword, written_type);
        body.handing = false;
        let mut typed = typed?;
        let source_copied = match self.expression_type_quietly(body, inner) {
            Some(source) => source.is_copied(),
            None => typed.value_type.is_copied(),
        };
        if source_copied {
            return Err(self.error(Diagnostic::refused_fix(
                marked,
                written,
                "plain numbers and `bool` are always copied, and take no word (D2, D123)",
            )));
        }
        if ownership == syntax::Ownership::Give
            && let Some((local, path)) = place
            && let Some(root) = root_name(inner)
        {
            self.only_lent(body, local, root, &path, inner.span)?;
            body.take_read(local);
            let site = body.site(root.span);
            let marker = body.calls.last().copied().unwrap_or(marked);
            body.events.push(crate::flow::Event::Give {
                place: crate::flow::Place { local, path, shown: written },
                spot: crate::flow::Spot { variable: root.span, place: inner.span, site, marker },
            });
        }
        typed.variable = None;
        typed.handed = true;
        Ok(typed)
    }

    /// The type of a variable or field, without checking it again or recording anything.
    pub fn expression_type_quietly(&self, body: &Body, expression: &syntax::Expression) -> Option<Type> {
        match &expression.kind {
            ExpressionKind::Variable(name) => {
                body.lookup(&name.text).map(|variable| variable.value_type.read_as().clone())
            }
            ExpressionKind::Group(inner) => self.expression_type_quietly(body, inner),
            ExpressionKind::Field { .. } => {
                let (local, path, _) = self.field_place(body, expression)?;
                let mut current =
                    body.variables.iter().rev().find(|variable| variable.local == local)?.value_type.read_as().clone();
                for index in path {
                    let Type::Struct { id, .. } = current else { return None };
                    current = self.aggregates.structs.get(id)?.fields.get(index)?.value_type.clone();
                }
                Some(current)
            }
            _ => None,
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
        if let ExpressionKind::As { value: inner, keyword, written_type } = &value.kind {
            return self.owned_as(body, ownership, ownership_span, value, inner, keyword, written_type);
        }
        if ownership == syntax::Ownership::Give && is_item(value) {
            return Err(self.error(Diagnostic::not_designed(
                value.span,
                "giving away one item of a collection",
                NOT_DECIDED,
            )));
        }
        let typed = self.expression(body, value, expected)?;
        if typed.value_type.is_copied() && self.instance {
            return Ok(typed);
        }
        if typed.value_type.is_copied() {
            let written = self.slice(value.span).to_string();
            return Err(self.error(Diagnostic::refused_fix(
                ownership_span.to(value.span),
                written,
                "plain numbers and `bool` are always copied, and take no word (D2, D123)",
            )));
        }
        if ownership == syntax::Ownership::Copy && body.purity != super::Purity::Plain {
            return Err(self.error(Diagnostic::not_designed(
                value.span,
                "whether a `fn.pure` may allocate, as `Copy` can",
                "not decided yet (standard 12.5)",
            )));
        }
        if ownership == syntax::Ownership::Give
            && let Some((local, path)) = self.any_place(body, value)
            && let Some(root) = root_name(value)
        {
            self.only_lent(body, local, root, &path, value.span)?;
            body.take_read(local);
            let site = body.site(root.span);
            let marker = body.calls.last().copied().unwrap_or(ownership_span.to(value.span));
            let shown = self.slice(value.span).to_string();
            body.events.push(crate::flow::Event::Give {
                place: crate::flow::Place { local, path, shown },
                spot: crate::flow::Spot { variable: root.span, place: value.span, site, marker },
            });
        }
        match ownership {
            syntax::Ownership::Give | syntax::Ownership::Copy => {
                let mut typed = typed;
                typed.variable = None;
                typed.handed = true;
                Ok(typed)
            }
            _ => Err(self.error(Diagnostic::refused(
                ownership_span.to(value.span),
                format!("`{}` here needs a stored borrow's type", ownership.word()),
                "a stored borrow is a type, such as `Lend from $r reading` (D2)",
            ))),
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
        if let Some(result) = self.collection_value(body, expression, into) {
            return result;
        }
        // A literal lands in a `type.new` as what it's made from (D141).
        if let Type::New { inner, .. } = into
            && (Self::is_literal_number(expression)
                || matches!(expression.kind, ExpressionKind::Text(_) | ExpressionKind::Bool(_)))
        {
            return self.value_into(body, expression, inner);
        }
        let typed = self.expression(body, expression, Some(into))?;
        if !typed.value_type.is_copied() && !typed.handed && is_held(expression) && !typed.value_type.is_unknown() {
            let shown = self.slice(expression.span).to_string();
            if let Some(root) = root_variable(expression)
                && body.lookup(&root).is_some_and(|variable| {
                    matches!(variable.value_type, Type::Failure { .. })
                        // A lent type parameter can only be copied out (D244).
                        || (matches!(variable.value_type, Type::Param { .. }) && body.lent.contains(&variable.local))
                })
            {
                return Err(self.error(Diagnostic::refused_fix(
                    expression.span,
                    format!("Copy {shown}"),
                    "every hand-off shows on the page, including one into a name, a `return` or a field (D3)",
                )));
            }
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
        let mut typed = typed;
        if typed.value_type.is_unknown() || into.is_unknown() {
            return Ok(typed.value);
        }
        // A unit has to match where it's stored, as written or once expanded (D100, D101, D242).
        // Between different number types, it's the type that's wrong, which the arms below say (D7).
        if matches!(into, Type::Integer { .. } | Type::Float { .. })
            && matches!(typed.value_type, Type::Integer { .. } | Type::Float { .. })
            && (typed.value_type.unit().is_some() || into.unit().is_some())
            && same_number(&typed.value_type, into)
        {
            // A label that carries a unit gives it to a plain number, and refuses another unit (D144).
            if let Some(carried) = &self.label_unit
                && into.unit() == Some(carried)
            {
                match typed.value_type.unit() {
                    None => {}
                    Some(unit) if !self.same_unit(Some(unit), Some(carried)) => {
                        return Err(self.error(Diagnostic::refused(
                            span,
                            format!("this is `{}`, and the label asks for `{into}`", typed.value_type),
                            "a label's unit takes a plain number or that unit, and a value in another is refused (D144)",
                        )));
                    }
                    Some(_) => {}
                }
            }
            let literal = typed.literal.is_some() || typed.float_literal.is_some();
            let takes_mark = literal
                && typed.value_type.unit().is_some_and(|unit| super::units::unit_mark(unit).is_none())
                && typed.value_type.unit().and_then(super::units::unmarked)
                    == into.unit().and_then(super::units::unmarked);
            let carried = self.label_unit.is_some() && into.unit() == self.label_unit.as_ref();
            if !carried && !takes_mark && !self.same_unit(typed.value_type.unit(), into.unit()) {
                return Err(self.unit_mismatch(span, &typed, into));
            }
            // The units match, so the number goes on to the checks below with the unit it lands in.
            typed.value_type = typed.value_type.with_unit(into.unit().cloned());
        }
        match (&typed.value_type, into) {
            (
                Type::Integer { kind: from, bounds: from_bounds, .. },
                Type::Integer { kind: to, bounds: to_bounds, .. },
            ) if from == to => {
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
                Type::Float { bits: from_bits, kind: from_kind, bounds: from_bounds, .. },
                Type::Float { bits: to_bits, kind: to_kind, bounds: to_bounds, .. },
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
            (Type::Str(from), Type::Str(to)) if from != to => {
                // trace: D326, D327, D369
                match (from, to) {
                    (_, StrSize::View) => Ok(typed.value),
                    (StrSize::View, StrSize::Growing) if matches!(typed.value.kind, Kind::Text(_)) => {
                        let shown = self.slice(span).to_string();
                        Err(self.error(Diagnostic::refused_fix(
                            span,
                            format!("str.from({shown}, alloc: Lend $heap)"),
                            "anything that takes heap memory takes an allocator, so heap text comes from a call like `str.from(…, alloc: Lend $heap)` (D321, D326, D328, D369)",
                        )))
                    }
                    (_, StrSize::Bytes(most)) => {
                        if let Kind::Text(text) = &typed.value.kind
                            && text.len() as u64 > *most
                        {
                            return Err(self.error(Diagnostic::refused(
                                span,
                                format!("this text is {} bytes, and `{into}` holds {most}", text.len()),
                                "a string's `len.max` is in bytes, and a literal that can't fit is a compile error (D326, D20)",
                            )));
                        }
                        Ok(typed.value)
                    }
                    _ => Ok(typed.value),
                }
            }
            (from, to) if from == to && !matches!(from, Type::Integer { .. }) => Ok(typed.value),
            (from, to) => {
                let why = if matches!(from, Type::New { .. }) || matches!(to, Type::New { .. }) {
                    NEW_WHY
                } else {
                    CONVERT_WHY
                };
                Err(self.error(Diagnostic::refused(span, format!("this is `{from}`, where `{to}` is needed"), why)))
            }
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
pub fn is_held(expression: &syntax::Expression) -> bool {
    match &expression.kind {
        ExpressionKind::Variable(_) => true,
        ExpressionKind::Field { base, .. } => is_held(base),
        ExpressionKind::Bracket { base, .. } if !matches!(base.kind, ExpressionKind::Path(_)) => is_held(base),
        ExpressionKind::Group(inner) => is_held(inner),
        _ => false,
    }
}

/// Where a held value lives: its variable's local, and the fields down to it, by their numbers.
pub fn place_of(body: &Body, expression: &syntax::Expression) -> Option<(checked::LocalId, Vec<usize>)> {
    match &expression.kind {
        ExpressionKind::Variable(name) => body.lookup(&name.text).map(|variable| (variable.local, Vec::new())),
        ExpressionKind::Group(inner) => place_of(body, inner),
        _ => None,
    }
}

/// A method's signature, found by its type and name with either mark.
fn signature_for(checker: &Checker<'_>, owner: &str, bare: &str) -> (String, super::Signature) {
    for name in [format!("{owner}.%{bare}"), format!("{owner}.%%{bare}")] {
        if let Some(signature) = checker.functions.get(&name) {
            return (name, signature.clone());
        }
    }
    (String::new(), super::Signature::default())
}

/// The variable a held value is in, through any fields.
fn root_variable(expression: &syntax::Expression) -> Option<String> {
    match &expression.kind {
        ExpressionKind::Variable(name) => Some(name.text.clone()),
        ExpressionKind::Field { base, .. } | ExpressionKind::Group(base) => root_variable(base),
        _ => None,
    }
}

/// The variable a place starts from, as written: `$r` in `$r.celsius`.
pub fn root_name(expression: &syntax::Expression) -> Option<&syntax::Name> {
    match &expression.kind {
        ExpressionKind::Variable(name) => Some(name),
        ExpressionKind::Field { base, .. } | ExpressionKind::Group(base) => root_name(base),
        ExpressionKind::Bracket { base, .. } if !matches!(base.kind, ExpressionKind::Path(_)) => root_name(base),
        _ => None,
    }
}

/// Whether two number types are the same, apart from their bounds and units.
fn same_number(a: &Type, b: &Type) -> bool {
    match (a, b) {
        (Type::Integer { kind: x, .. }, Type::Integer { kind: y, .. }) => x == y,
        (Type::Float { bits: x, kind: p, .. }, Type::Float { bits: y, kind: q, .. }) => x == y && p == q,
        _ => false,
    }
}

/// Whether a value is one item of a list or array, as in `$a[pos: 1]`.
pub fn is_item(expression: &syntax::Expression) -> bool {
    match &expression.kind {
        ExpressionKind::Bracket { base, .. } => !matches!(base.kind, ExpressionKind::Path(_)),
        ExpressionKind::Group(inner) => is_item(inner),
        _ => false,
    }
}

/// Whether a value is of a struct or union, which gets operators only by defining them (D236).
fn is_own_type(value_type: &Type) -> bool {
    matches!(value_type, Type::Struct { .. } | Type::Union { .. })
}
