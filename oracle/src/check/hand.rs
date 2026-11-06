//! Ownership words: what `Give`, `Copy`, `Lend` and `Lend.mut` do, and where a word is needed.

use super::body::{root_variable, walk};
use super::{Checker, Origin, ParameterInfo};
use crate::ast::{Expression, ExpressionKind, HandWord, ParameterWord};
use crate::ir::{self, Node, NodeKind, Place};
use crate::report::{Outcome, Report, ReportExtra, Span};
use crate::types::Type;

/// A value that lives in a variable or a constant, as an ownership word sees it.
pub(super) struct Held {
    pub node: Node,
    pub place: Option<Place>,
    pub value_type: Type,
    pub root_name: String,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum UseKind {
    Read,
    Write,
}

/// One word written in a call, for checking two arguments against each other.
pub(super) struct HandedOff {
    pub root: String,
    pub path: Vec<usize>,
    pub word: HandWord,
    pub span: Span,
}

impl Checker<'_> {
    /// Where a problem with a use is shown: the whole call around it when there is one, as 11.1 does.
    pub(super) fn use_span(&self, span: Span) -> Span {
        self.context.call_span.unwrap_or(span)
    }

    /// trace: D2
    pub(super) fn require_alive(&self, slot: usize, span: Span) -> Outcome<()> {
        let Some(variable) = self.context.variables.get(slot) else {
            return Ok(());
        };
        if let Some(given) = self.context.state.get(slot).and_then(|state| state.given) {
            return Err(Report::refused(
                self.use_span(span),
                format!("`${}` was already given away", variable.name),
                "a given value is dead afterwards (D2)",
            )
            .because(given, format!("`${}` was given away here", variable.name)));
        }
        Ok(())
    }

    /// Tells any stored borrow of `$name` that it's been used, in a way that may end the borrow.
    /// Whether a borrow lasts to its last use or to its scope isn't decided, so nothing is said
    /// until the borrow is read again, which is wrong either way. (D2)
    pub(super) fn note_use(&mut self, name: &str, kind: UseKind, span: Span) {
        let conflicts: Vec<usize> = self
            .context
            .variables
            .iter()
            .enumerate()
            .filter(|(slot, variable)| {
                variable.origin == Origin::StoredBorrow
                    && variable.borrows.iter().any(|borrowed| borrowed == name)
                    && (variable.mutable || kind == UseKind::Write)
                    && self.is_in_scope(*slot)
            })
            .map(|(slot, _)| slot)
            .collect();
        for slot in conflicts {
            if let Some(state) = self.context.state.get_mut(slot) {
                state.invalidated = state.invalidated.or(Some(span));
            }
        }
    }

    fn is_in_scope(&self, slot: usize) -> bool {
        self.context
            .scopes
            .iter()
            .any(|scope| scope.values().any(|found| *found == slot))
    }

    fn refuse_invalidated(&self, slot: usize, span: Span) -> Outcome<()> {
        let Some(variable) = self.context.variables.get(slot) else {
            return Ok(());
        };
        if let Some(used) = self
            .context
            .state
            .get(slot)
            .and_then(|state| state.invalidated)
        {
            return Err(Report::refused(
                span,
                format!("`${}` can't be used after what it borrows was used", variable.name),
                "a stored `Lend.mut` means nothing else uses it meanwhile, and a `Lend` means it can't change (D2)",
            )
            .because(used, "what it borrows was used here"));
        }
        Ok(())
    }

    /// Reads a variable or a field of one, for an ownership word or to take its value.
    pub(super) fn check_held(&mut self, expression: &Expression) -> Outcome<Held> {
        match &expression.kind {
            ExpressionKind::Variable(name) => self.check_held_variable(name, expression.span),
            ExpressionKind::Field { base, name } => {
                let held = self.check_held(base)?;
                let (index, value_type) = self.find_field(base, &held.value_type, name)?;
                let mut place = held.place;
                if let Some(place) = &mut place {
                    place.path.push(index);
                }
                let node = match &place {
                    Some(place) => Node {
                        span: expression.span,
                        kind: NodeKind::Read(place.clone()),
                    },
                    None => Node {
                        span: expression.span,
                        kind: NodeKind::Field(Box::new(held.node), index),
                    },
                };
                Ok(Held {
                    node,
                    place,
                    value_type,
                    root_name: held.root_name,
                })
            }
            _ => Err(Report::refused(
                expression.span,
                "an ownership word goes on a variable or a field, and a fresh value takes no word",
                "a fresh value that has no holder yet takes no word (D2)",
            )),
        }
    }

    fn check_held_variable(&mut self, name: &str, span: Span) -> Outcome<Held> {
        if let Some(slot) = self.find_variable(name) {
            let variable = self.context.variables.get(slot).cloned().ok_or_else(|| {
                Report::refused(span, format!("`${name}` isn't declared"), "(D57)")
            })?;
            // In compile-time code a local constant is its value, since no frame exists yet.
            if self.context.in_constant > 0 {
                if let Some(constant) = &variable.constant {
                    return Ok(Held {
                        node: constant.clone(),
                        place: None,
                        value_type: variable.held_type,
                        root_name: name.to_string(),
                    });
                }
            }
            self.require_alive(slot, span)?;
            self.refuse_invalidated(slot, span)?;
            self.note_use(name, UseKind::Read, span);
            let value_type = match &variable.held_type {
                Type::Borrow { inner, .. } => (**inner).clone(),
                other => other.clone(),
            };
            let place = Place {
                slot,
                path: Vec::new(),
            };
            return Ok(Held {
                node: Node {
                    span,
                    kind: NodeKind::Read(place.clone()),
                },
                place: Some(place),
                value_type,
                root_name: name.to_string(),
            });
        }
        if let Some(global) = self.globals.get(name) {
            let node = if self.context.in_constant > 0 {
                global.node.clone()
            } else {
                Node {
                    span,
                    kind: NodeKind::Global(global.index),
                }
            };
            return Ok(Held {
                node,
                place: None,
                value_type: global.value_type.clone(),
                root_name: name.to_string(),
            });
        }
        // trace: D57
        Err(Report::refused(
            span,
            format!("`${name}` isn't declared"),
            "`let` and `const` declare a variable with its type and value (D57, D83)",
        ))
    }

    /// Finds a field on a struct, or on a union while the test trusts its case. (D9, D57, D104)
    pub(super) fn find_field(
        &self,
        base: &Expression,
        base_type: &Type,
        name: &crate::ast::Name,
    ) -> Outcome<(usize, Type)> {
        match base_type {
            Type::Struct(struct_name) => {
                let fields = self
                    .structs
                    .get(struct_name)
                    .map(|info| info.fields.as_slice())
                    .unwrap_or_default();
                fields
                    .iter()
                    .position(|(field, _)| *field == name.text)
                    .and_then(|index| Some((index, fields.get(index)?.1.clone())))
                    .ok_or_else(|| {
                        Report::refused(
                            name.span,
                            format!("`{struct_name}` has no field `{}`", name.text),
                            "a field read is a plain `.` (D57)",
                        )
                    })
            }
            Type::Union(union_name) => self.find_union_field(base, union_name, name),
            other => Err(Report::refused(
                name.span,
                format!("`{other}` has no field `{}`", name.text),
                "a field read is a plain `.` (D57)",
            )),
        }
    }

    fn find_union_field(
        &self,
        base: &Expression,
        union_name: &str,
        name: &crate::ast::Name,
    ) -> Outcome<(usize, Type)> {
        let cases = self
            .unions
            .get(union_name)
            .map(|info| info.cases.as_slice())
            .unwrap_or_default();
        let shown = self.text_of(base.span).to_string();
        let slot = match &base.kind {
            ExpressionKind::Variable(variable) => self.find_variable(variable),
            _ => None,
        };
        let state = slot.and_then(|slot| self.context.state.get(slot));
        if let Some(case) = state.and_then(|state| state.trust) {
            let Some((case_name, fields)) = cases.get(case) else {
                return Err(Report::refused(
                    name.span,
                    "this case doesn't exist",
                    "(D33)",
                ));
            };
            if let Some(index) = fields.iter().position(|(field, _)| *field == name.text) {
                let field_type = fields
                    .get(index)
                    .map_or(Type::Void, |field| field.1.clone());
                return Ok((index, field_type));
            }
            return Err(Report::refused(
                name.span,
                format!("`{union_name}.{case_name}` has no field `{}`", name.text),
                "a case's values go in `[ ]` (D33)",
            ));
        }
        let Some((case_name, _)) = cases
            .iter()
            .find(|(_, fields)| fields.iter().any(|(field, _)| *field == name.text))
        else {
            return Err(Report::refused(
                name.span,
                format!("no case of `{union_name}` has a field `{}`", name.text),
                "a case's values go in `[ ]` (D33)",
            ));
        };
        // trace: D104
        if let Some(ended) = state.and_then(|state| state.trust_ended) {
            return Err(Report::refused(
                base.span.to(name.span),
                format!("`{shown}` may not be `{union_name}.{case_name}` any more"),
                "a test's trust ends at a `change`, and at a `Lend.mut` too (D104)",
            )
            .because(ended, format!("`{shown}` changed here")));
        }
        // trace: D9
        Err(Report::refused(
            base.span.to(name.span),
            format!("`{shown}` isn't known to be `{union_name}.{case_name}` here"),
            "inside `if $s: status.failed:` the compiler trusts it, and outside that's a compile error (D9)",
        ))
    }

    /// `Give $x`: the value moves, and `$x` is dead afterwards on any path. (D2)
    fn check_give(&mut self, value: &Expression, hand_span: Span) -> Outcome<Held> {
        if !matches!(value.kind, ExpressionKind::Variable(_)) {
            if matches!(value.kind, ExpressionKind::Field { .. }) {
                return Err(Report::not_designed(
                    value.span,
                    "giving away one field of a struct",
                    "",
                ));
            }
            return Err(Report::refused(
                value.span,
                "a fresh value takes no word",
                "a fresh value that has no holder yet takes no word (D2)",
            ));
        }
        let held = self.check_held(value)?;
        let Some(slot) = held.place.as_ref().map(|place| place.slot) else {
            return Err(Report::not_designed(
                value.span,
                "giving away a constant",
                "",
            ));
        };
        let (origin, name) = {
            let variable = self.context.variables.get(slot);
            (
                variable.map(|variable| variable.origin),
                variable.map_or(String::new(), |variable| variable.name.clone()),
            )
        };
        // trace: D2
        match origin {
            Some(Origin::Owned | Origin::GivenParameter) => {}
            Some(Origin::Constant) => {
                return Err(Report::not_designed(
                    value.span,
                    "giving away a constant",
                    "",
                ));
            }
            _ => {
                return Err(Report::refused(
                    value.span,
                    format!("`${name}` is only lent here, so it can't be given away"),
                    "`Lend` is temporary access, and the caller keeps it (D2)",
                ));
            }
        }
        self.refuse_while_read(&name, value.span)?;
        self.note_use(&name, UseKind::Write, value.span);
        let given_span = self.use_span(hand_span);
        if let Some(state) = self.context.state.get_mut(slot) {
            state.given = Some(given_span);
            state.given_on_some_path = false;
        }
        Ok(held)
    }

    /// Nothing may change or take `$xs` while `for` is reading it in place. (D2)
    pub(super) fn refuse_while_read(&self, name: &str, span: Span) -> Outcome<()> {
        if self.context.being_read.iter().any(|read| read == name) {
            return Err(Report::refused(
                span,
                format!("a `for` is reading `${name}` in place here"),
                "`Lend.mut` means nothing else uses it meanwhile, and `for` is using it (D2)",
            ));
        }
        Ok(())
    }

    fn check_copy(&mut self, value: &Expression) -> Outcome<Held> {
        let held = self.check_held(value)?;
        if self.is_must_give(&held.value_type) {
            return Err(Report::not_designed(
                value.span,
                "`Copy` on a `must_Give` value, which would make a second duty",
                "",
            ));
        }
        Ok(held)
    }

    /// A place for `Lend` or `Lend.mut`.
    pub(super) fn check_lent_place(&mut self, value: &Expression, word: HandWord) -> Outcome<Held> {
        let held = self.check_held(value)?;
        if word != HandWord::LendMut {
            return Ok(held);
        }
        let Some(slot) = held.place.as_ref().map(|place| place.slot) else {
            return Err(Report::refused(
                value.span,
                "a constant can't be lent with `Lend.mut`",
                "a constant's value is fixed at compile time (D83)",
            ));
        };
        let Some(variable) = self.context.variables.get(slot).cloned() else {
            return Ok(held);
        };
        // trace: D3
        if !variable.mutable {
            return Err(Report::refused(
                value.span,
                format!(
                    "`${}` can't change, so it can't be lent with `Lend.mut`",
                    variable.name
                ),
                "`let` declares a variable that can't change, and `let mut` one that can (D3)",
            ));
        }
        if variable.origin == Origin::StoredBorrow {
            return Err(Report::not_designed(
                value.span,
                "lending through a stored `Lend.mut`",
                "12.8",
            ));
        }
        self.refuse_while_read(&variable.name, value.span)?;
        self.note_use(&variable.name, UseKind::Write, value.span);
        // trace: D104
        let ended_at = self.use_span(value.span);
        if let Some(state) = self.context.state.get_mut(slot) {
            if state.trust.is_some() {
                state.trust = None;
                state.trust_ended = Some(ended_at);
            }
        }
        Ok(held)
    }

    /// A value stored into a name, a `return`, a field or an item: it must show its hand-off. (D3)
    pub(super) fn check_into(&mut self, value: &Expression, target: &Type) -> Outcome<Node> {
        if let ExpressionKind::Hand { word, value: inner } = &value.kind {
            // trace: D2
            if target.is_always_copied() {
                return Err(Self::word_on_a_copied_value(
                    value,
                    self.text_of(inner.span),
                    target,
                ));
            }
            let held = match word {
                HandWord::Give => self.check_give(inner, value.span)?,
                HandWord::Copy => self.check_copy(inner)?,
                HandWord::Lend | HandWord::LendMut => {
                    return Err(Report::refused(
                        value.span,
                        format!("`{}` here needs a stored borrow's type", word.word()),
                        "a stored borrow is a type, such as `Lend from $r reading` (D2)",
                    ));
                }
            };
            Self::check_held_type(value.span, target, &held.value_type)?;
            return Ok(held.node);
        }
        if root_variable(value).is_some()
            && matches!(
                value.kind,
                ExpressionKind::Variable(_) | ExpressionKind::Field { .. }
            )
        {
            let (node, found) = self.check_expression(value, Some(target))?;
            // trace: D3
            if !found.is_always_copied() {
                return Err(Report::refused(
                    value.span,
                    format!("`{}` is `{found}`, so this needs `Give` or `Copy`", self.text_of(value.span)),
                    "every hand-off shows on the page, including one into a name, a `return` or a field (D3)",
                ));
            }
            return self.coerce(node, &found, target, value);
        }
        let (node, found) = self.check_expression(value, Some(target))?;
        self.coerce(node, &found, target, value)
    }

    fn word_on_a_copied_value(
        expression: &Expression,
        shown: &str,
        value_type: &Type,
    ) -> Box<Report> {
        Report::refused(
            expression.span,
            format!("`{value_type}` is always copied, and takes no word"),
            "plain numbers and `bool` are always copied, and take no word (D2, D123)",
        )
        .fix(shown)
    }

    /// Whole values handed around must be the same type. Which list bounds fit which isn't stated.
    pub(super) fn check_held_type(span: Span, target: &Type, found: &Type) -> Outcome<()> {
        if target == found {
            return Ok(());
        }
        if let (Type::List { item: left, .. }, Type::List { item: right, .. }) = (target, found) {
            if left == right {
                return Err(Report::not_designed(
                    span,
                    format!("whether `{found}` is accepted where `{target}` is asked for"),
                    "",
                ));
            }
        }
        Err(Self::mismatch(span, target, found))
    }

    /// One argument, against the parameter it fills. (D2)
    pub(super) fn check_argument(
        &mut self,
        value: &Expression,
        parameter: &ParameterInfo,
        handed: &mut Vec<HandedOff>,
    ) -> Outcome<ir::Argument> {
        let target = &parameter.parameter_type;
        if target.is_always_copied() {
            // trace: D2
            if let ExpressionKind::Hand { value: inner, .. } = &value.kind {
                return Err(Self::word_on_a_copied_value(
                    value,
                    self.text_of(inner.span),
                    target,
                ));
            }
            let (node, found) = self.check_expression(value, Some(target))?;
            return Ok(ir::Argument::Value(
                self.coerce(node, &found, target, value)?,
            ));
        }
        let wanted = match parameter.word {
            ParameterWord::Bare => HandWord::Lend,
            ParameterWord::LentMut => HandWord::LendMut,
            ParameterWord::Given => HandWord::Give,
        };
        if let ExpressionKind::Hand { word, value: inner } = &value.kind {
            let matches_parameter = *word == wanted
                || (parameter.word == ParameterWord::Given && *word == HandWord::Copy);
            if !matches_parameter {
                return Err(self.wrong_word(value, inner, *word, wanted));
            }
            let held = match word {
                HandWord::Give => self.check_give(inner, value.span)?,
                HandWord::Copy => self.check_copy(inner)?,
                HandWord::Lend | HandWord::LendMut => self.check_lent_place(inner, *word)?,
            };
            Self::check_held_type(value.span, target, &held.value_type)?;
            handed.push(HandedOff {
                root: held.root_name.clone(),
                path: held
                    .place
                    .as_ref()
                    .map(|place| place.path.clone())
                    .unwrap_or_default(),
                word: *word,
                span: value.span,
            });
            return Ok(match (*word, held.place) {
                (HandWord::Lend | HandWord::LendMut, Some(place)) => ir::Argument::Alias(place),
                _ => ir::Argument::Value(held.node),
            });
        }
        if matches!(
            value.kind,
            ExpressionKind::Variable(_) | ExpressionKind::Field { .. }
        ) && root_variable(value).is_some()
        {
            // trace: D3
            let shown = self.text_of(value.span).to_string();
            let found = self.check_held(value)?.value_type;
            if found.is_always_copied() {
                return Err(Self::mismatch(value.span, target, &found));
            }
            let report = Report::refused(
                value.span,
                format!("`{shown}` is passed whole, so this call says `Give` or `Copy`"),
                "every hand-off shows on the page (D3)",
            );
            return Err(match parameter.word {
                ParameterWord::Bare => report.fix(format!("Lend {shown}")),
                ParameterWord::LentMut => report.fix(format!("Lend.mut {shown}")),
                ParameterWord::Given => report,
            });
        }
        if parameter.word == ParameterWord::LentMut {
            return Err(Report::not_designed(
                value.span,
                "lending a fresh value with `Lend.mut`",
                "",
            ));
        }
        let (node, found) = self.check_expression(value, Some(target))?;
        Ok(ir::Argument::Value(
            self.coerce(node, &found, target, value)?,
        ))
    }

    fn wrong_word(
        &self,
        value: &Expression,
        inner: &Expression,
        word: HandWord,
        wanted: HandWord,
    ) -> Box<Report> {
        let shown = self.text_of(inner.span);
        let report = Report::refused(
            value.span,
            format!("this parameter doesn't take `{}`", word.word()),
            "the call and the parameter both say it, and they must match (D2)",
        );
        match wanted {
            HandWord::Lend | HandWord::LendMut => report.fix(format!("{} {shown}", wanted.word())),
            // `Give` and `Copy` both fit, so there's no one fix to show. (D16)
            HandWord::Give | HandWord::Copy => {
                let mut report = report;
                report.what =
                    "this parameter is `Given`, so the call says `Give` or `Copy`".to_string();
                report
            }
        }
    }

    /// Two arguments in one call can't get in each other's way. (D2)
    pub(super) fn check_handed_together(
        handed: &[HandedOff],
        reads: &[(String, Span)],
    ) -> Outcome<()> {
        for (index, first) in handed.iter().enumerate() {
            for second in handed.iter().skip(index + 1) {
                if first.root != second.root {
                    continue;
                }
                let pair = (first.word, second.word);
                let overlaps =
                    first.path.starts_with(&second.path) || second.path.starts_with(&first.path);
                let definite = matches!(
                    pair,
                    (HandWord::Give, _)
                        | (_, HandWord::Give)
                        | (HandWord::LendMut, HandWord::Lend | HandWord::LendMut)
                        | (HandWord::Lend, HandWord::LendMut)
                ) && !matches!(pair, (HandWord::Copy, _) | (_, HandWord::Copy));
                if definite && overlaps {
                    return Err(Report::refused(
                        second.span,
                        format!(
                            "`${}` is already handed off with `{}` in this call",
                            first.root,
                            first.word.word()
                        ),
                        "`Lend.mut` means nothing else uses it meanwhile, and `Give` means it's gone (D2)",
                    )
                    .because(first.span, "that's here"));
                }
                let troubled = matches!(pair, (HandWord::LendMut | HandWord::Give, _))
                    || matches!(pair, (_, HandWord::LendMut | HandWord::Give));
                if troubled {
                    return Err(Report::not_designed(
                        second.span,
                        format!(
                            "`{}` and `{}` on parts of `${}` in one call",
                            first.word.word(),
                            second.word.word(),
                            first.root
                        ),
                        "",
                    ));
                }
            }
        }
        for read in reads {
            if handed
                .iter()
                .any(|hand| hand.word == HandWord::LendMut && hand.root == read.0)
            {
                return Err(Report::not_designed(
                    read.1,
                    format!(
                        "reading `${}` in a call that lends it with `Lend.mut`",
                        read.0
                    ),
                    "",
                ));
            }
        }
        Ok(())
    }

    /// The variables an argument reads with no word on them.
    pub(super) fn reads_in_place(arguments: &[&Expression]) -> Vec<(String, Span)> {
        let mut reads = Vec::new();
        for argument in arguments {
            // A word on the argument itself is `handed`, so it's only what's inside others.
            if let ExpressionKind::Hand { .. } = argument.kind {
                continue;
            }
            walk(argument, &mut |part| {
                if let ExpressionKind::Variable(name) = &part.kind {
                    reads.push((name.clone(), part.span));
                }
            });
        }
        reads
    }
}
