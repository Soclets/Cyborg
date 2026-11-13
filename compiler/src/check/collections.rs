//! Lists and arrays (D11, D12, D24, D102, D133, D140, D142): their first values, their length, getting and changing
//! an item, and adding one to a list.

use crate::checked::{self, ExpressionKind as Kind, MachineType};
use crate::diagnostic::{Diagnostic, Kind as DiagnosticKind};
use crate::source::Span;
use crate::syntax::{self, ExpressionKind};

use super::expressions::{Typed, When, failure};
use super::types::{IntegerKind, Type, Unit};
use super::{Body, Check, Checker, NOT_DECIDED, Origin, Reported};

const POSITION_WHY: &str =
    "`pos:` counts from 1, and `offset.items:` counts from 0, so the word says which you mean (D133, D144)";

/// A place being read or changed: its variable, the fields and items down to it, and its type.
pub struct Reached {
    pub root: super::Variable,
    pub root_span: Span,
    pub steps: Vec<checked::Step>,
    pub value_type: Type,
}

impl Checker<'_> {
    /// A list's or array's first value: `[1, 2, 3]`, `all 0` or `empty`, which take their type from where they land
    /// (D102, D103). `None` when the value is something else.
    pub fn collection_value(
        &mut self,
        body: &mut Body,
        expression: &syntax::Expression,
        into: &Type,
    ) -> Option<Check<checked::Expression>> {
        let machine_type = into.machine();
        match (&expression.kind, into) {
            (ExpressionKind::List(items), Type::Array { item, len, .. }) => Some((|| {
                if items.len() as u64 != *len {
                    return Err(self.error(Diagnostic::refused(
                        expression.span,
                        format!("`{into}` holds exactly {len} items, and this gives {}", items.len()),
                        "`[ ]` gives an array its items, with the count checked against the type (D102)",
                    )));
                }
                let values = self.items_into(body, items, item)?;
                Ok(checked::Expression { kind: Kind::Build(values), machine_type })
            })()),
            (ExpressionKind::List(items), Type::List { item, min, max, len_type, .. }) => Some((|| {
                let count = items.len() as u64;
                if max.is_none() {
                    // trace: D369
                    let shown = self.slice(expression.span).to_string();
                    return Err(self.error(Diagnostic::refused_fix(
                        expression.span,
                        format!("std::coll::list.from({shown}, alloc: Lend $heap)"),
                        "a list that grows on the heap is filled with `std::coll::list.from([…], alloc: Lend $heap)`, since every allocation names its allocator (D321, D369)",
                    )));
                }
                let (_, most) = len_type.range();
                if max.is_none() && i128::from(count) > most {
                    return Err(self.error(Diagnostic::not_designed(
                        expression.span,
                        format!("a list of `inf` with more items than its `{}` can count", len_type.name()),
                        NOT_DECIDED,
                    )));
                }
                if count < *min || max.is_some_and(|max| count > max) {
                    return Err(self.error(Diagnostic::refused(
                        expression.span,
                        format!("`{into}` can't hold {count} items"),
                        "`[ ]` gives a list its items, with the count checked against the type (D102)",
                    )));
                }
                let values = self.items_into(body, items, item)?;
                Ok(checked::Expression { kind: Kind::ListOf(values), machine_type })
            })()),
            (ExpressionKind::Empty, Type::List { view, .. }) if *view != super::types::ListView::Owned => {
                // trace: D379
                Some(Err(self.error(Diagnostic::refused(
                    expression.span,
                    format!("`{into}` can't be `empty`"),
                    "`empty` is refused where it can't be true (D103)",
                ))))
            }
            (ExpressionKind::Empty, Type::List { min, .. }) => Some(if *min > 0 {
                Err(self.error(Diagnostic::refused(
                    expression.span,
                    format!("`{into}` can't be `empty`"),
                    "`empty` is refused where it can't be true, so a list with `len.min` above 0 can't use it (D102, D103)",
                )))
            } else {
                Ok(checked::Expression { kind: Kind::ListOf(Vec::new()), machine_type })
            }),
            (ExpressionKind::All(_), Type::List { .. }) => Some(Err(self.error(Diagnostic::refused(
                expression.span,
                "`all` fills an array",
                "`= all 0` fills an array with one value (D102)",
            )))),
            (ExpressionKind::All(value), Type::Array { item, len, .. }) => Some((|| {
                let value = self.value_into(body, value, item)?;
                Ok(checked::Expression { kind: Kind::Fill { value: Box::new(value), len: *len }, machine_type })
            })()),
            (ExpressionKind::Empty, Type::Str(_)) => {
                Some(Ok(checked::Expression { kind: Kind::Text(String::new()), machine_type }))
            }
            _ => None,
        }
    }

    fn items_into(
        &mut self,
        body: &mut Body,
        items: &[syntax::Expression],
        item: &Type,
    ) -> Check<Vec<checked::Expression>> {
        let mut values = Vec::new();
        let mut failed = false;
        for each in items {
            match self.value_into(body, each, item) {
                Ok(value) => values.push(value),
                Err(Reported) => failed = true,
            }
        }
        if failed { Err(Reported) } else { Ok(values) }
    }

    /// `$xs.len`: how many items, in its `len.type`, with the unit `items` (D140).
    pub fn length(&mut self, typed: &Typed) -> Option<Typed> {
        let items = Some(Unit::named("items"));
        match &typed.value_type {
            Type::Array { len, len_type, .. } => {
                let value_type = Type::Integer { kind: *len_type, bounds: None, unit: items };
                let mut result = Typed::new(value_type, Kind::Integer(i128::from(*len)));
                result.literal = None;
                Some(result)
            }
            Type::List { len_type, .. } => {
                let count = checked::Expression {
                    kind: Kind::ListLen(Box::new(typed.value.clone())),
                    machine_type: MachineType::Integer { bits: 64, signed: false },
                };
                let value_type = Type::Integer { kind: *len_type, bounds: None, unit: items };
                let kind = if *len_type == IntegerKind::U64 { count.kind } else { Kind::Convert(Box::new(count)) };
                Some(Typed::new(value_type, kind))
            }
            _ => None,
        }
    }

    /// The step to one item: its position, checked against the count (D133, D142).
    fn item_step(
        &mut self,
        body: &mut Body,
        expression: &syntax::Expression,
        base_text: &str,
        labels: &[syntax::Label],
        collection: &Type,
    ) -> Check<(checked::Step, Type)> {
        let item = match collection {
            Type::Array { item, .. } | Type::List { item, .. } => (**item).clone(),
            Type::Unknown => return Err(Reported),
            other => {
                return Err(self.error(Diagnostic::refused(
                    expression.span,
                    format!("`pos:` and `offset:` work on lists and arrays, and this is `{other}`"),
                    "both work on lists and arrays (D133)",
                )));
            }
        };
        // A missing or other label is said at the `[`.
        let opening = expression.span.start as usize + base_text.len();
        let opening = Span::new(self.file, opening, opening + 1);
        let [label] = labels else {
            return Err(self.error(Diagnostic::refused(
                opening,
                "getting an item says `pos:` or `offset.items:`",
                POSITION_WHY,
            )));
        };
        let word = label.name.text();
        if label.sets {
            let at = self.text[label.name.span.end as usize..].find('=').unwrap_or(0) + label.name.span.end as usize;
            return Err(self.error(Diagnostic::refused_fix(
                Span::new(self.file, at, at + 1),
                ":",
                "a label names a value inside brackets (D13)",
            )));
        }
        let from_one = match word.as_str() {
            "pos" => true,
            "offset.items" => false,
            "offset" => {
                return Err(self.error(Diagnostic::refused_fix(
                    label.name.span,
                    "offset.items",
                    "an offset could read as bytes, so there's no bare `offset:` (D143, D144)",
                )));
            }
            _ if word.starts_with("offset.") => {
                return Err(self.error(Diagnostic::refused_fix(
                    label.name.span,
                    "offset.items",
                    "a label's unit takes a plain number or that unit, and a value in another is refused (D144)",
                )));
            }
            _ if word.starts_with("pos.") => {
                return Err(self.error(Diagnostic::not_designed(label.name.span, format!("`{word}:`"), NOT_DECIDED)));
            }
            _ => {
                return Err(self.error(Diagnostic::refused(
                    opening,
                    "getting an item says `pos:` or `offset.items:`",
                    POSITION_WHY,
                )));
            }
        };
        let mark = body.events.len();
        let position = self.expression(body, &label.value, None)?;
        if body.events[mark.min(body.events.len())..]
            .iter()
            .any(|event| matches!(event, crate::flow::Event::Give { .. }))
        {
            return Err(self.error(Diagnostic::not_designed(
                label.value.span,
                "handing something off inside a position",
                NOT_DECIDED,
            )));
        }
        match &position.value_type {
            Type::Integer { unit: None, .. } => {}
            Type::Integer { unit: Some(unit), .. } if unit.single() == Some("items") => {}
            Type::Integer { unit: Some(unit), .. } | Type::Float { unit: Some(unit), .. } => {
                return Err(self.error(Diagnostic::refused(
                    label.value.span,
                    format!("a position is plain or in `items`, and this is in `{unit}`"),
                    "`pos:` and `offset.items:` take a plain number or one in `items` (D142, D144)",
                )));
            }
            Type::Unknown => return Err(Reported),
            other => {
                return Err(self.error(Diagnostic::refused(
                    label.value.span,
                    format!("a position is a whole number, and this is `{other}`"),
                    "`pos:` counts from 1, and `offset.items:` counts from 0 (D133, D144)",
                )));
            }
        }
        let check = failure(
            self,
            Diagnostic::new(
                DiagnosticKind::CheckFailed,
                expression.span,
                format!("`{word}: \u{1}` is outside `{base_text}`"),
                "a bad position is a bounds check (D26, D133)",
            ),
            &[When::Text("it holds ".into()), When::Operand(1), When::Text(" items".into())],
        );
        Ok((checked::Step::Item { position: Box::new(position.value), from_one, check: Some(Box::new(check)) }, item))
    }

    /// A place written as a variable with fields and items after it, or `None` when it isn't one.
    pub fn reach(&mut self, body: &mut Body, expression: &syntax::Expression) -> Check<Option<Reached>> {
        match &expression.kind {
            ExpressionKind::Variable(name) => {
                let Some(root) = body.lookup(&name.text).cloned() else { return Ok(None) };
                if body.changing.iter().any(|each| *each == name.text) {
                    return Err(self.error(Diagnostic::refused(
                        name.span,
                        format!("a `for` is changing `{}` through its items here", name.text),
                        "`Lend.mut` means nothing else uses it meanwhile, and the `for` is using it (D2, D180)",
                    )));
                }
                let value_type = root.value_type.read_as().clone();
                Ok(Some(Reached { root, root_span: name.span, steps: Vec::new(), value_type }))
            }
            ExpressionKind::Group(inner) => self.reach(body, inner),
            ExpressionKind::Field { base, field } => {
                let Some(mut reached) = self.reach(body, base)? else { return Ok(None) };
                let Type::Struct { id, .. } = &reached.value_type else { return Ok(None) };
                let info = self.aggregates.structs[*id].clone();
                let Some(index) = info.fields.iter().position(|each| each.name == field.text) else { return Ok(None) };
                reached.steps.push(checked::Step::Field(index));
                reached.value_type = info.fields[index].value_type.read_as().clone();
                Ok(Some(reached))
            }
            ExpressionKind::Bracket { base, labels, .. } if !matches!(base.kind, ExpressionKind::Path(_)) => {
                let Some(mut reached) = self.reach(body, base)? else { return Ok(None) };
                let base_text = self.slice(base.span).to_string();
                let (step, item) = self.item_step(body, expression, &base_text, labels, &reached.value_type)?;
                reached.steps.push(step);
                reached.value_type = item;
                Ok(Some(reached))
            }
            _ => Ok(None),
        }
    }

    /// `$xs[pos: 1]`: one item, read in place (D133).
    pub fn item_value(
        &mut self,
        body: &mut Body,
        expression: &syntax::Expression,
        base: &syntax::Expression,
        labels: &[syntax::Label],
    ) -> Check<Typed> {
        if let Some(reached) = self.reach(body, expression)? {
            self.read_root(body, &reached);
            let place = checked::Place { local: reached.root.local, steps: reached.steps };
            let mut typed = Typed::new(reached.value_type, Kind::Load(place));
            // A check on where it goes names it as written (D20).
            typed.variable = Some(self.slice(expression.span).to_string());
            return Ok(typed);
        }
        // A fresh value, like `make()[pos: 2]`, goes in a temporary first.
        let typed = self.expression(body, base, None)?;
        let base_text = self.slice(base.span).to_string();
        let (step, item) = self.item_step(body, expression, &base_text, labels, &typed.value_type)?;
        let local = body.new_local(typed.value.machine_type.clone());
        let load = checked::Expression {
            kind: Kind::Load(checked::Place { local, steps: vec![step] }),
            machine_type: item.machine(),
        };
        Ok(Typed::new(item, Kind::Bind { local, value: Box::new(typed.value), body: Box::new(load) }))
    }

    fn read_root(&self, body: &mut Body, reached: &Reached) {
        if !reached.root.value_type.is_copied() && reached.root.origin != Origin::Constant {
            let site = body.site(reached.root_span);
            body.events.push(crate::flow::Event::Read {
                place: crate::flow::Place { local: reached.root.local, path: Vec::new(), shown: String::new() },
                spot: crate::flow::Spot { variable: reached.root_span, place: reached.root_span, site, marker: site },
            });
        }
    }

    /// `change $xs[pos: 3] = 10;`, maybe through fields: what changing a field needs, at an item (D125, D133).
    pub fn change_item(
        &mut self,
        body: &mut Body,
        span: Span,
        target: &syntax::Expression,
        value: &syntax::Expression,
        block: &mut checked::Block,
    ) -> Check<()> {
        let Some(root_name) = super::expressions::root_name(target) else {
            return Err(self.not_yet(target.span, "`change` on this".to_string()));
        };
        let Some(root) = body.lookup(&root_name.text).cloned() else {
            if self.constants.contains_key(&root_name.text) {
                return Err(self.error(Diagnostic::refused(
                    target.span,
                    format!("`{}` is a constant, so it can't change", root_name.text),
                    "a constant's value is fixed at compile time (D83)",
                )));
            }
            return Err(self.error(Diagnostic::refused(
                root_name.span,
                format!("`{}` isn't declared", root_name.text),
                "`let` declares a variable (D2)",
            )));
        };
        if !root.mutable {
            return Err(self.error(Diagnostic::refused(
                target.span,
                format!("`{}` can't change, so neither can its item", root_name.text),
                "a field or item can be changed if the whole variable is `let mut`, or the parameter `Lent.mut` (D125, D133)",
            )));
        }
        self.not_while_reading(body, &root_name.text, target.span)?;
        let Some(reached) = self.reach(body, target)? else {
            return Err(self.not_yet(target.span, "`change` on this".to_string()));
        };
        let mark = body.events.len();
        let written_value_span = value.span;
        let value = self.value_into(body, value, &reached.value_type)?;
        if body.events[mark.min(body.events.len())..]
            .iter()
            .any(|event| matches!(event, crate::flow::Event::LendMut { place, .. } if place.local == root.local))
        {
            return Err(self.error(Diagnostic::not_designed(
                written_value_span,
                format!("changing part of `{}` with a value that lends it out", root_name.text),
                NOT_DECIDED,
            )));
        }
        body.events.push(crate::flow::Event::Change {
            place: crate::flow::Place { local: root.local, path: Vec::new(), shown: root_name.text.clone() },
            spot: crate::flow::Spot {
                variable: root_name.span,
                place: target.span,
                site: root_name.span,
                marker: span,
            },
        });
        block.push(checked::Statement::Store {
            place: checked::Place { local: root.local, steps: reached.steps },
            value,
        });
        Ok(())
    }

    /// A list or array a `for` reads in place can't change meanwhile (D2, D128).
    pub fn not_while_reading(&mut self, body: &Body, name: &str, span: Span) -> Check<()> {
        if body.reading_parts.iter().any(|each| each == name) {
            return Err(self.error(Diagnostic::not_designed(
                span,
                format!("changing one part of `{name}` while a `for` reads another"),
                NOT_DECIDED,
            )));
        }
        if body.reading.iter().any(|each| each == name) {
            return Err(self.error(Diagnostic::refused(
                span,
                format!("a `for` is reading `{name}` in place here"),
                "`Lend.mut` means nothing else uses it meanwhile, and `for` is using it (D2)",
            )));
        }
        Ok(())
    }

    /// `$xs.std::%%append(val: 4)`: one more item at the end, if the list has room (D24), or with
    /// `alloc: Lend $heap` for a list that grows (D321, D322). `?? void` ignores a full list or one out of memory,
    /// and `otherwise:` handles running out of memory (D96, D324).
    #[allow(clippy::too_many_lines)]
    pub fn append(
        &mut self,
        body: &mut Body,
        expression: &syntax::Expression,
        receiver: &syntax::Expression,
        arguments: &[syntax::Argument],
        handling: &super::memory::Handling<'_>,
        block: &mut checked::Block,
    ) -> Check<(checked::Statement, Option<checked::LocalId>)> {
        use super::memory::{Handling, OutOfMemoryHandling};
        let Some(root_name) = super::expressions::root_name(receiver) else {
            return Err(self.not_yet(receiver.span, "adding to a fresh list".to_string()));
        };
        let Some(root) = body.lookup(&root_name.text).cloned() else { return Err(Reported) };
        if !root.mutable {
            return Err(self.error(Diagnostic::refused(
                receiver.span,
                format!("`{}` can't change, so it can't be lent with `Lend.mut`", root_name.text),
                "`let` declares a variable that can't change, and `let mut` one that can (D3)",
            )));
        }
        self.not_while_reading(body, &root_name.text, receiver.span)?;
        let Some(reached) = self.reach(body, receiver)? else {
            return Err(self.not_yet(receiver.span, "adding to this".to_string()));
        };
        let list_type = reached.value_type.clone();
        let Type::List { item, max, len_type, view, .. } = &list_type else {
            return Err(self.not_yet(receiver.span, format!("`append` on `{}`", reached.value_type)));
        };
        if *view != super::types::ListView::Owned {
            // trace: D379
            return Err(self.error(Diagnostic::refused(
                expression.span,
                format!("`{list_type}` is a view, so it can't grow or shrink"),
                "a list with no `len.max` is a view onto someone else's list: `.mut` changes its items, and only the list itself grows (D379)",
            )));
        }
        let (argument, allocator) = match arguments {
            [argument] => (argument, None),
            [argument, allocator] => (argument, Some(allocator)),
            _ => return Err(self.not_yet(expression.span, "`append` with these arguments".to_string())),
        };
        if argument.label.as_ref().map(|label| label.text.as_str()) != Some("val") {
            return Err(self.not_yet(argument.span, "`append` without `val:`".to_string()));
        }
        // A full bounded list fails, and what that failure is called isn't decided (standard 12.7).
        if max.is_some()
            && let Handling::Arms(_) = handling
        {
            return Err(self.error(Diagnostic::not_designed(
                expression.span,
                "what the failure is called when a list is full",
                "not decided yet (standard 12.7)",
            )));
        }
        if max.is_some() && body.purity != super::Purity::Plain {
            return Err(self.error(Diagnostic::not_designed(
                expression.span,
                "whether a `fn.pure` may fail, as appending to a full list does",
                "not decided yet (standard 12.5)",
            )));
        }
        let flag = match handling {
            Handling::Arms(_) => {
                let local = body.new_local(checked::MachineType::Bool);
                block.push(checked::Statement::Let {
                    local,
                    value: checked::Expression {
                        kind: checked::ExpressionKind::Bool(false),
                        machine_type: checked::MachineType::Bool,
                    },
                });
                Some(local)
            }
            _ => None,
        };
        let out_of_memory = match (handling, flag) {
            (Handling::Ignore, _) => OutOfMemoryHandling::Ignore,
            (_, Some(local)) => OutOfMemoryHandling::Flag(local),
            _ => OutOfMemoryHandling::Crash,
        };
        let call = match &expression.kind {
            syntax::ExpressionKind::Instead { value, .. } | syntax::ExpressionKind::Otherwise { value, .. } => {
                value.as_ref()
            }
            _ => expression,
        };
        let growth = self.append_growth(body, &list_type, call, allocator, out_of_memory)?;
        let value = self.value_into(body, &argument.value, item)?;
        // A list with room for at most `len.max`, or as many as its `len.type` counts, fails past it.
        let shown = self.slice(receiver.span).to_string();
        let room = match max {
            Some(_) if matches!(handling, Handling::Ignore) => max.map(|max| (max, None)),
            Some(max) => {
                let check = failure(
                    self,
                    Diagnostic::new(
                        DiagnosticKind::CheckFailed,
                        call.span,
                        format!("`{shown}` is full, and nothing handled it"),
                        "a failure nothing handles stops the program (D96)",
                    ),
                    &[When::Text("it holds ".into()), When::Operand(0), When::Text(" items".into())],
                );
                Some((*max, Some(Box::new(check))))
            }
            None if *len_type != IntegerKind::U64 => {
                let (_, most) = len_type.range();
                let check = failure(
                    self,
                    Diagnostic::new(
                        DiagnosticKind::NotDesignedYet,
                        expression.span,
                        format!("`{shown}` growing past what its `len.type` can count, with `inf` as its `len.max`"),
                        NOT_DECIDED,
                    ),
                    &[],
                );
                Some((u64::try_from(most).unwrap_or(u64::MAX), Some(Box::new(check))))
            }
            None => None,
        };
        let site = body.site(root_name.span);
        body.events.push(crate::flow::Event::LendMut {
            place: crate::flow::Place { local: root.local, path: Vec::new(), shown: root_name.text.clone() },
            spot: crate::flow::Spot { variable: root_name.span, place: receiver.span, site, marker: site },
        });
        let push = checked::Statement::Push {
            place: checked::Place { local: root.local, steps: reached.steps },
            value,
            room,
            growth,
        };
        Ok((push, flag))
    }
}
