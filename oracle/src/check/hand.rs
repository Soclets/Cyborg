//! Ownership words: what `Give`, `Copy`, `Lend` and `Lend.mut` do, and where a word is needed.

use super::body::{root_variable, walk};
use super::{Checker, Origin, ParameterInfo};
use crate::ast::{Expression, ExpressionKind, HandWord, ParameterWord};
use crate::ir::{self, Node, NodeKind, Place, Step};
use crate::report::{Outcome, Report, ReportExtra, Span};
use crate::types::{Sources, Type};

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
        // Every path assigns a `decl` before any read. (D203)
        // trace: D203
        let filled = self
            .context
            .state
            .get(slot)
            .map_or(super::Filled::Yes, |state| state.filled);
        // trace: D215
        if filled != super::Filled::Yes && self.context.skipped.contains(&slot) {
            return Err(Report::refused(
                self.use_span(span),
                format!("`${}` never got a value", variable.name),
                "a variable is read only after it has one (D203)",
            ));
        }
        if filled != super::Filled::Yes {
            return Err(Report::refused(
                self.use_span(span),
                if filled == super::Filled::No {
                    format!("`${}` has no value yet", variable.name)
                } else {
                    format!("`${}` has no value yet on some path here", variable.name)
                },
                "every path has to `assign` a `decl` before reading it (D203)",
            )
            .because(
                variable.declared,
                format!("`${}` is declared here", variable.name),
            ));
        }
        if let Some(given) = self.context.state.get(slot).and_then(|state| state.given) {
            if self
                .context
                .state
                .get(slot)
                .is_some_and(|state| state.finished)
            {
                return Err(Report::not_designed(
                    self.use_span(span),
                    format!("using `${}` after `finish`", variable.name),
                    "",
                ));
            }
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
        self.note_use_at(name, &[], kind, span);
    }

    /// The same, for a use of one part of `$name`: different fields are separate. (D173)
    pub(super) fn note_use_at(&mut self, name: &str, path: &[usize], kind: UseKind, span: Span) {
        let conflicts: Vec<usize> = self
            .context
            .variables
            .iter()
            .enumerate()
            .filter(|(slot, variable)| {
                // A struct holding borrows only reads what it borrows. (D171)
                let ended_by_this = if variable.origin == Origin::StoredBorrow {
                    variable.mutable || kind == UseKind::Write
                } else {
                    kind == UseKind::Write
                };
                variable.borrows.iter().any(|borrowed| borrowed == name)
                    && ended_by_this
                    && ir::paths_overlap(path, &variable.borrowed_path)
                    && self.is_in_scope(*slot)
            })
            .map(|(slot, _)| slot)
            .collect();
        for slot in conflicts {
            // trace: D169
            if self.context.held_borrows.contains(&slot) && self.context.held_conflict.is_none() {
                self.context.held_conflict = Some((span, slot));
            }
            if let Some(state) = self.context.state.get_mut(slot) {
                state.invalidated = state.invalidated.or(Some(span));
            }
        }
    }

    /// The values a borrow of `$name` really borrows: through a stored borrow, the original. (D174)
    pub(super) fn borrow_roots(&self, name: &str) -> Vec<String> {
        let Some(slot) = self.find_variable(name) else {
            return vec![name.to_string()];
        };
        match self.context.variables.get(slot) {
            Some(variable) if variable.origin == Origin::StoredBorrow => variable
                .borrows
                .iter()
                .filter(|borrowed| {
                    self.find_variable(borrowed)
                        .and_then(|found| self.context.variables.get(found))
                        .is_none_or(|found| found.origin != Origin::StoredBorrow)
                })
                .cloned()
                .collect(),
            _ => vec![name.to_string()],
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
                    place.path.push(Step::Field(index));
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
            ExpressionKind::Item {
                base,
                position,
                counts_from_zero,
            } => {
                let held = self.check_held(base)?;
                let (value_type, position) = self.check_position(
                    base,
                    &held.value_type,
                    position,
                    *counts_from_zero,
                    expression.span,
                )?;
                let mut place = held.place;
                let node = if let Some(place) = &mut place {
                    place.path.push(Step::Item(Box::new(position)));
                    Node {
                        span: expression.span,
                        kind: NodeKind::Read(place.clone()),
                    }
                } else {
                    Node {
                        span: expression.span,
                        kind: NodeKind::Item(Box::new(held.node), Box::new(position)),
                    }
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

    /// What `[pos: …]` gets from a collection, and its position, which is a plain number.
    /// trace: D133
    pub(super) fn check_position(
        &mut self,
        base: &Expression,
        base_type: &Type,
        position: &Expression,
        counts_from_zero: bool,
        span: Span,
    ) -> Outcome<(Type, ir::Position)> {
        let item_type = match base_type {
            Type::List { item, .. } | Type::Array { item, .. } => (**item).clone(),
            Type::Map { .. } => {
                return Err(Report::not_designed(span, "getting an item from a map", ""));
            }
            Type::New { .. } => {
                return Err(Report::not_designed(
                    span,
                    "getting an item from a `type.new`, rather than crossing with `as` first",
                    "12.1",
                ));
            }
            other => {
                return Err(Report::refused(
                    span,
                    format!("`pos:` and `offset:` work on lists and arrays, and this is `{other}`"),
                    "both work on lists and arrays (D133)",
                ));
            }
        };
        let mut gives_away = false;
        walk(position, &mut |part| {
            gives_away = gives_away
                || matches!(
                    part.kind,
                    ExpressionKind::Hand {
                        word: HandWord::Give | HandWord::LendMut,
                        ..
                    }
                );
        });
        if gives_away {
            return Err(Report::not_designed(
                position.span,
                "handing something off inside a position",
                "",
            ));
        }
        let (value, position_type) = self.check_expression(
            position,
            Some(&Type::plain_integer(crate::types::IntegerKind::I32)),
        )?;
        match &position_type {
            Type::Integer { unit: None, .. } => {}
            // trace: D142
            Type::Integer {
                unit: Some(unit), ..
            } if unit == "items" => {}
            Type::Integer {
                unit: Some(unit), ..
            } => {
                return Err(Report::refused(
                    position.span,
                    format!("a position is plain or in `items`, and this is in `{unit}`"),
                    "`pos:` and `offset.items:` take a plain number or one in `items` (D142, D144)",
                ));
            }
            other => {
                return Err(Report::refused(
                    position.span,
                    format!("a position is a whole number, and this is `{other}`"),
                    "`pos:` counts from 1, and `offset.items:` counts from 0 (D133, D144)",
                ));
            }
        }
        Ok((
            item_type,
            ir::Position {
                value,
                counts_from_zero,
                shown: self.text_of(base.span).to_string(),
                span,
            },
        ))
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
            self.refuse_after_a_move(slot, name, span)?;
            // trace: D180
            if self
                .context
                .being_changed
                .iter()
                .any(|changed| changed == name)
            {
                return Err(Report::refused(
                    span,
                    format!("a `for` is changing `${name}` through its items here"),
                    "`Lend.mut` means nothing else uses it meanwhile, and the `for` is using it (D2, D180)",
                ));
            }
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
            // Reading through a borrow, or into a struct holding one, with its sources filled in.
            // trace: D171
            Type::Borrow { inner, .. } => self.find_field(base, inner, name),
            Type::Holding {
                base: inner,
                sources,
                fills,
            } => {
                let (index, found) = self.find_field(base, inner, name)?;
                Ok((index, fill_sources(&found, sources, fills)))
            }
            Type::Struct(struct_name) => {
                let fields = self
                    .struct_info(struct_name)
                    .map(|info| info.fields)
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
            // trace: D139
            Type::Failure {
                name: failure,
                fields,
            } => fields
                .iter()
                .position(|(field, _)| *field == name.text)
                .and_then(|index| Some((index, fields.get(index)?.1.clone())))
                .ok_or_else(|| {
                    Report::refused(
                        name.span,
                        format!("`{failure}` carries no `{}`", name.text),
                        "a failure can carry data in `[ ]`, and `case negative $f:` reads it (D47, D139)",
                    )
                }),
            Type::New { .. } => Err(Report::not_designed(
                name.span,
                "a field of a `type.new`, rather than crossing with `as` first",
                "12.1",
            )),
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
        // D139 reads a case's data through the arm's name, and doesn't say whether this still can.
        if state.is_some_and(|state| state.matched) {
            return Err(Report::not_designed(
                base.span.to(name.span),
                format!("reading `{shown}.{}` in a `match` arm, rather than through a name for the case", name.text),
                "",
            ));
        }
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
    pub(super) fn check_give(&mut self, value: &Expression, hand_span: Span) -> Outcome<Held> {
        if matches!(value.kind, ExpressionKind::Field { .. }) && root_variable(value).is_some() {
            return self.check_give_field(value, hand_span);
        }
        if !matches!(value.kind, ExpressionKind::Variable(_)) {
            if matches!(value.kind, ExpressionKind::Item { .. }) {
                return Err(Report::not_designed(
                    value.span,
                    "giving away one item of a collection",
                    "",
                ));
            }
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
        // trace: D325
        if let Some(promised) = self
            .context
            .state
            .get(slot)
            .and_then(|state| state.promised)
        {
            return Err(Report::refused(
                value.span,
                format!("`${name}` is promised to a `defer`, so it can't be given away here"),
                "a deferred `Give` ends the value's duty when its block ends, so the value is yours to read and change until then, but not to give away (D325)",
            )
            .because(promised, "it's promised here"));
        }
        self.refuse_while_read(&name, &[], value.span)?;
        self.note_use(&name, UseKind::Write, value.span);
        let given_span = self.use_span(hand_span);
        if let Some(state) = self.context.state.get_mut(slot) {
            state.given = Some(given_span);
            state.given_on_some_path = false;
        }
        Ok(held)
    }

    /// `Give $l.file` moves a field out of something this code owns, and afterwards `$l` can only
    /// have more fields moved out, or be finished. (D185)
    /// trace: D185
    fn check_give_field(&mut self, value: &Expression, hand_span: Span) -> Outcome<Held> {
        let root = root_variable(value).unwrap_or_default();
        let slot = self.find_variable(&root);
        let origin = slot
            .and_then(|slot| self.context.variables.get(slot))
            .map(|variable| variable.origin);
        if !matches!(origin, Some(Origin::Owned | Origin::GivenParameter)) {
            return Err(Report::refused(
                value.span,
                format!("`${root}` is only borrowed here, so a field can't be moved out of it"),
                "a field moves out only of something you own, a `Given` parameter or your own `let` (D185)",
            ));
        }
        self.context.moving_field = true;
        let checked = self.check_held(value);
        self.context.moving_field = false;
        let held = checked?;
        let Some(place) = held.place.clone() else {
            return Err(Report::not_designed(
                value.span,
                "giving away a constant",
                "",
            ));
        };
        let path = place.keys();
        if path.contains(&ir::ANY_ITEM) {
            return Err(Report::not_designed(
                value.span,
                "giving away one item of a collection",
                "",
            ));
        }
        let earlier = self.context.state.get(place.slot).and_then(|state| {
            state
                .moved
                .iter()
                .find(|moved| ir::paths_overlap(&moved.path, &path))
                .map(|moved| moved.span)
        });
        if let Some(earlier) = earlier {
            return Err(Report::refused(
                value.span,
                format!("`{}` was already moved out", self.text_of(value.span)),
                "a given value is dead afterwards (D2, D185)",
            )
            .because(earlier, "it was moved out here"));
        }
        self.refuse_while_read(&root, &path, value.span)?;
        self.note_use_at(&root, &path, UseKind::Write, value.span);
        let span = self.use_span(hand_span);
        if let Some(state) = self.context.state.get_mut(place.slot) {
            state.moved.push(super::MovedField {
                path,
                span,
                every_path: true,
            });
        }
        Ok(held)
    }

    /// After a field is moved out, `$l` can only have more moved out, or be finished. (D185)
    fn refuse_after_a_move(&self, slot: usize, name: &str, span: Span) -> Outcome<()> {
        if self.context.moving_field {
            return Ok(());
        }
        let Some(moved) = self
            .context
            .state
            .get(slot)
            .and_then(|state| state.moved.first())
        else {
            return Ok(());
        };
        Err(Report::refused(
            span,
            format!("a field of `${name}` was moved out, so all that's left is moving out another, or `finish`"),
            "after a field is moved out, the value is incomplete (D185)",
        )
        .because(moved.span, "it was moved out here"))
    }

    /// Nothing may change or take `$xs` while `for` is reading it in place. (D2)
    pub(super) fn refuse_while_read(&self, name: &str, path: &[usize], span: Span) -> Outcome<()> {
        for (root, read_path) in &self.context.being_read {
            if root != name {
                continue;
            }
            if ir::paths_overlap(path, read_path) {
                return Err(Report::refused(
                    span,
                    format!("a `for` is reading `${name}` in place here"),
                    "`Lend.mut` means nothing else uses it meanwhile, and `for` is using it (D2)",
                ));
            }
            return Err(Report::not_designed(
                span,
                format!("changing one part of `${name}` while a `for` reads another"),
                "",
            ));
        }
        Ok(())
    }

    pub(super) fn check_copy(&mut self, value: &Expression) -> Outcome<Held> {
        if matches!(
            self.context.kind,
            crate::ast::FunctionKind::Pure | crate::ast::FunctionKind::PureBaked
        ) {
            return Err(Report::not_designed(
                value.span,
                "whether a `fn.pure` may allocate, as `Copy` can",
                "12.5",
            ));
        }
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
        // trace: D174
        if !variable.mutable && variable.origin == Origin::StoredBorrow {
            return Err(Report::refused(
                value.span,
                format!(
                    "`${}` only reads what it borrows, so it can't lend it with `Lend.mut`",
                    variable.name
                ),
                "a stored `Lend` can be lent on with `Lend` only (D174, D2)",
            ));
        }
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
        // A stored `Lend.mut` can be lent on, and the new borrow names the original. (D174)
        let path = held.place.as_ref().map(ir::Place::keys).unwrap_or_default();
        self.refuse_while_read(&variable.name, &path, value.span)?;
        self.note_use_at(&variable.name, &path, UseKind::Write, value.span);
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
                ExpressionKind::Variable(_)
                    | ExpressionKind::Field { .. }
                    | ExpressionKind::Item { .. }
            )
        {
            let (node, found) = self.check_expression(value, Some(target))?;
            // trace: D3
            if !found.is_always_copied() {
                let refused = Report::refused(
                    value.span,
                    format!("`{}` is `{found}`, so this needs `Give` or `Copy`", self.text_of(value.span)),
                    "every hand-off shows on the page, including one into a name, a `return` or a field (D3)",
                );
                // Something lent can't be given away, so `Copy` is the one word that fits. (D244)
                // trace: D244
                let lent = root_variable(value)
                    .and_then(|name| self.find_variable(&name))
                    .and_then(|slot| self.context.variables.get(slot))
                    .is_some_and(|variable| {
                        matches!(variable.origin, Origin::LentParameter | Origin::ForItem)
                    });
                if lent {
                    return Err(refused.fix(format!("Copy {}", self.text_of(value.span))));
                }
                return Err(refused);
            }
            return self.coerce(node, &found, target, value);
        }
        let (node, found) = self.check_expression(value, Some(target))?;
        self.coerce(node, &found, target, value)
    }

    pub(super) fn word_on_a_copied_value(
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
        // Lending any string gives a view of it, so a `str` parameter reads every kind. (D327)
        // trace: D327
        if *target == Type::Str && matches!(found, Type::BoundedStr(_) | Type::GrowableStr) {
            return Ok(());
        }
        // A parameter filled with the function's own sources takes any value of that type,
        // whose sources those then stand for. (D171)
        if let (
            Type::Holding {
                base: wanted,
                fills: wanted_fills,
                ..
            },
            Type::Holding { base: given, .. },
        ) = (target, found)
        {
            if wanted == given && wanted_fills.iter().all(|fill| fill.starts_with('`')) {
                return Ok(());
            }
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

    /// A number lent to change says `Lend.mut`, and that's the one word it takes. (D184)
    /// trace: D184
    fn check_number_lent_to_change(
        &mut self,
        value: &Expression,
        parameter: &ParameterInfo,
        handed: &mut Vec<HandedOff>,
    ) -> Outcome<ir::Argument> {
        let target = &parameter.parameter_type;
        let ExpressionKind::Hand {
            word: HandWord::LendMut,
            value: inner,
        } = &value.kind
        else {
            let mut report = Report::refused(
                value.span,
                format!("`{}` changes this, so it says `Lend.mut`", parameter_name(parameter)),
                "a number lent to change says `Lend.mut`, and it's the one word a number takes (D184, D3)",
            );
            let place = match &value.kind {
                ExpressionKind::Hand { value: inner, .. } => inner,
                _ => value,
            };
            if root_variable(place).is_some() {
                report = report.fix(format!("Lend.mut {}", self.text_of(place.span)));
            }
            return Err(report);
        };
        let held = self.check_lent_place(inner, HandWord::LendMut)?;
        Self::check_held_type(value.span, target, &held.value_type)?;
        let Some(place) = held.place else {
            return Err(Report::refused(
                value.span,
                "a constant can't be lent with `Lend.mut`",
                "a constant's value is fixed at compile time (D83)",
            ));
        };
        handed.push(HandedOff {
            root: held.root_name.clone(),
            path: place.keys(),
            word: HandWord::LendMut,
            span: value.span,
        });
        Ok(ir::Argument::Alias(place))
    }

    /// One argument, against the parameter it fills. (D2)
    pub(super) fn check_argument(
        &mut self,
        value: &Expression,
        parameter: &ParameterInfo,
        handed: &mut Vec<HandedOff>,
    ) -> Outcome<ir::Argument> {
        let target = &parameter.parameter_type;
        if parameter.label_has_unit {
            return self.check_argument_for_unit_label(value, parameter);
        }
        if target.is_always_copied() && parameter.word == ParameterWord::LentMut {
            return self.check_number_lent_to_change(value, parameter, handed);
        }
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
                path: held.place.as_ref().map(ir::Place::keys).unwrap_or_default(),
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
            ExpressionKind::Variable(_)
                | ExpressionKind::Field { .. }
                | ExpressionKind::Item { .. }
        ) && root_variable(value).is_some()
        {
            // trace: D3
            let shown = self.text_of(value.span).to_string();
            let found = self.check_held(value)?.value_type;
            if found.is_always_copied() {
                return Err(Self::mismatch(value.span, target, &found));
            }
            // trace: D139
            if matches!(found, Type::Failure { .. }) {
                return Err(Report::refused(
                    value.span,
                    format!("`{shown}` names a failure, which is for reading its data"),
                    "`case negative $f:` names the failure, so `$f.val` reads its data (D139)",
                ));
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

    /// `at.screen.pixels: 5`: a plain number takes the label's unit, a value with the right unit is
    /// fine, and a value with another is refused. (D144)
    /// trace: D144
    fn check_argument_for_unit_label(
        &mut self,
        value: &Expression,
        parameter: &ParameterInfo,
    ) -> Outcome<ir::Argument> {
        let target = &parameter.parameter_type;
        if let ExpressionKind::Hand { value: inner, .. } = &value.kind {
            return Err(Self::word_on_a_copied_value(
                value,
                self.text_of(inner.span),
                target,
            ));
        }
        let (node, found) = self.check_expression(value, Some(target))?;
        let with_unit = match (&found, target) {
            (
                Type::Integer {
                    kind,
                    bound,
                    unit: None,
                },
                Type::Integer { unit, .. },
            ) => Type::Integer {
                kind: *kind,
                bound: *bound,
                unit: unit.clone(),
            },
            (
                Type::Float {
                    kind,
                    bound,
                    unit: None,
                    rules,
                },
                Type::Float { unit, .. },
            ) => Type::Float {
                kind: *kind,
                bound: *bound,
                unit: unit.clone(),
                rules: *rules,
            },
            (other, _) => other.clone(),
        };
        let found_unit = match &with_unit {
            Type::Integer { unit, .. } | Type::Float { unit, .. } => unit.clone(),
            _ => None,
        };
        let target_unit = match target {
            Type::Integer { unit, .. } | Type::Float { unit, .. } => unit.clone(),
            _ => None,
        };
        if found_unit.is_some() && found_unit != target_unit {
            return Err(Report::refused(
                value.span,
                format!("this is `{found}`, and the label asks for `{target}`"),
                "a label's unit takes a plain number or that unit, and a value in another is refused (D144)",
            ));
        }
        Ok(ir::Argument::Value(
            self.coerce(node, &with_unit, target, value)?,
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
                let overlaps = ir::paths_may_meet(&first.path, &second.path);
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
                // Different fields are separate, so they can be handed off together. Items
                // overlap, since which one isn't known. (D173)
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

/// A field's type with its struct's sources replaced by what they're filled with. (D171)
pub(super) fn fill_sources(found: &Type, sources: &[String], fills: &[String]) -> Type {
    let fill_of = |source: &str| {
        sources
            .iter()
            .position(|name| name == source)
            .and_then(|index| fills.get(index))
            .cloned()
    };
    match found {
        Type::Borrow {
            mutable,
            sources: Sources::Named(source),
            inner,
        } => {
            let filled = match fill_of(source) {
                Some(fill) => match fill.strip_prefix('$') {
                    Some(name) => Sources::Variables(vec![name.to_string()]),
                    None if fill == "program" => Sources::Program,
                    None => Sources::Named(fill.trim_matches('`').to_string()),
                },
                None => Sources::Named(source.clone()),
            };
            Type::Borrow {
                mutable: *mutable,
                sources: filled,
                inner: inner.clone(),
            }
        }
        Type::Holding {
            base,
            sources: inner_sources,
            fills: inner_fills,
        } => Type::Holding {
            base: base.clone(),
            sources: inner_sources.clone(),
            fills: inner_fills
                .iter()
                .map(|fill| {
                    fill.strip_prefix('`')
                        .and_then(|rest| rest.strip_suffix('`'))
                        .and_then(fill_of)
                        .unwrap_or_else(|| fill.clone())
                })
                .collect(),
        },
        other => other.clone(),
    }
}

/// The struct or union under its filled-in sources. (D171, D176)
pub(super) fn plain_type(found: &Type) -> &Type {
    match found {
        Type::Holding { base, .. } => base,
        other => other,
    }
}

fn parameter_name(parameter: &ParameterInfo) -> String {
    format!("${}", parameter.name)
}
