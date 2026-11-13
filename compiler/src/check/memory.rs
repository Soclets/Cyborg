//! Explicit memory (D321 to D333, D369 to D371, D385 to D390): allocators, growable lists, `done`, and the
//! standard's sizes, which an arena counts in so it runs out at the same byte on every machine for a target.

use super::expressions::{When, failure, root_name};
use super::types::{AllocatorKind, Grow, StrSize, Type};
use super::{Body, Check, Checker, Reported};
use crate::checked::{self, ExpressionKind as Kind};
use crate::diagnostic::{Diagnostic, Kind as DiagnosticKind};
use crate::source::Span;
use crate::syntax::{self, ExpressionKind, Ownership};

const ALLOCATOR_WHY: &str = "anything that takes heap memory is handed an allocator, as in `alloc: Lend $heap`, so where memory comes from is always said (D321, D322)";
const WRONG_WHY: &str = "a list remembers which allocator it uses, and handing it another is a failed check (D322)";
const UNHANDLED_WHY: &str = "a failure nothing handles stops the program (D96)";

/// An item's size and alignment as the standard counts them (D370, D388, D389, D390), and whether they depend on
/// the target, as a pointer's do.
#[derive(Clone, Copy, Debug)]
pub struct Footprint {
    pub size: u64,
    pub align: u64,
    pub per_target: bool,
}

/// How an `append` or a `from` is handled when it fails.
pub enum Handling<'a> {
    /// Nothing handles it, so it crashes (D96).
    None,
    /// `?? void`.
    Ignore,
    /// `otherwise:` arms.
    Arms(&'a [syntax::Arm]),
}

impl Checker<'_> {
    /// `os::heap` and `std::mem::arena` as types (D332, D333).
    pub fn allocator_type(&mut self, text: &str, span: Span) -> Option<Check<Type>> {
        let kind = match text {
            "os::heap" => AllocatorKind::Heap,
            "std::mem::arena" => AllocatorKind::Arena,
            _ => return None,
        };
        Some(self.require_import(text, span).map(|()| Type::Allocator(kind)))
    }

    /// The standard's size and alignment of an item (D370): a type's own size at its natural alignment, a bare
    /// `struct` padded in its written order, `.compact` sorted by alignment, `.packed` with no padding, and a view as
    /// a pointer and a pointer-sized length (D390). `None` where the standard doesn't say.
    pub fn footprint(&self, value_type: &Type) -> Option<Footprint> {
        let pointer = self.target.pointer_bytes();
        let plain = |size: u64| Some(Footprint { size, align: size, per_target: false });
        match value_type {
            Type::Integer { kind, .. } => plain(u64::from(kind.bits.div_ceil(8))),
            Type::Float { bits, .. } => plain(u64::from(*bits / 8)),
            Type::Bool => plain(1),
            Type::Pointer { .. } => Some(Footprint { size: pointer, align: pointer, per_target: true }),
            Type::Str(StrSize::View) => Some(Footprint { size: 2 * pointer, align: pointer, per_target: true }),
            Type::New { inner, .. } => self.footprint(inner),
            Type::Simd { item, lanes } => {
                let item = self.footprint(item)?;
                Some(Footprint { size: item.size.checked_mul(*lanes)?, ..item })
            }
            Type::Array { item, len, .. } => {
                let item = self.footprint(item)?;
                Some(Footprint { size: item.size.checked_mul(*len)?, ..item })
            }
            // A bounded list holds its length, as its `len.type`, then room for every item (D389).
            Type::List { item, max: Some(max), len_type, .. } => {
                let item = self.footprint(item)?;
                let length = u64::from(len_type.bits.div_ceil(8));
                let align = item.align.max(length);
                let size = round_up(length, item.align)?.checked_add(item.size.checked_mul(*max)?)?;
                Some(Footprint { size: round_up(size, align)?, align, per_target: item.per_target })
            }
            Type::Struct { id, .. } => {
                let info = self.aggregates.structs.get(*id)?;
                let mut fields = Vec::new();
                for field in &info.fields {
                    fields.push(self.footprint(&field.value_type)?);
                }
                if info.layout == super::aggregates::Layout::Untagged {
                    // Its fields share the same bytes, so it's as big as its biggest (D398).
                    let size = fields.iter().map(|field| field.size).max().unwrap_or(0);
                    let align = fields.iter().map(|field| field.align).max().unwrap_or(1);
                    return Some(Footprint {
                        size: round_up(size, align)?,
                        align,
                        per_target: fields.iter().any(|field| field.per_target),
                    });
                }
                struct_footprint(&fields, info.layout)
            }
            _ => None,
        }
    }

    /// `alloc: Lend $heap`: an allocator lent to a call that allocates (D321, D322, D324).
    pub fn allocator_argument(
        &mut self,
        body: &mut Body,
        argument: &syntax::Argument,
    ) -> Check<(checked::Expression, AllocatorKind)> {
        if argument.label.as_ref().map(|label| label.text.as_str()) != Some("alloc") {
            return Err(self.not_yet(argument.span, "an allocator without `alloc:`".to_string()));
        }
        if argument.ownership.map(|(word, _)| word) != Some(Ownership::Lend) {
            return Err(self.not_yet(argument.span, "an allocator that isn't lent with `Lend`".to_string()));
        }
        let typed = self.expression(body, &argument.value, None)?;
        match typed.value_type {
            Type::Allocator(kind) => Ok((typed.value, kind)),
            Type::Unknown => Err(Reported),
            other => Err(self.error(Diagnostic::refused(
                argument.value.span,
                format!("this is `{other}`, where an allocator is needed"),
                "an allocator is `os::heap` or `std::mem::arena`, or anything that satisfies `std::mem::allocator` (D324, D332, D333)",
            ))),
        }
    }

    /// How a growable list of `item` grows with an allocator: its item's standard size, which an arena needs to
    /// know exactly (D370, D387, D389), and what a wrong allocator and running out of memory do.
    pub fn growth(
        &mut self,
        item: &Type,
        grow: Grow,
        allocator: (checked::Expression, AllocatorKind),
        span: Span,
        out_of_memory: OutOfMemoryHandling,
    ) -> Check<checked::Growth> {
        let (allocator, kind) = allocator;
        let footprint = self.footprint(item);
        let footprint = match footprint {
            Some(footprint) => footprint,
            // The heap never runs out at a counted byte, so only an arena needs the exact size.
            None if kind == AllocatorKind::Heap => Footprint { size: 1, align: 1, per_target: false },
            None => {
                return Err(self.not_yet(span, format!("counting how much `{item}` takes in an arena")));
            }
        };
        if kind == AllocatorKind::Arena && footprint.per_target && !crate::target::named() {
            // trace: D387, D389
            // It's about running, not checking, so the rest of the code is checked as if it ran.
            self.error(Diagnostic::refused(
                span,
                "how much these items take depends on the target, which this run doesn't name",
                "a pointer's size is the target's, so a program whose answer depends on it names its target with `--target=` (D387, D389)",
            ));
        }
        let (grow, step) = match grow {
            Grow::Double => (0, 1),
            Grow::Exact => (1, 1),
            Grow::ByItems(step) => (2, step),
            Grow::ByBytes(step) => (3, step),
        };
        let word = |index| When::Operand(index);
        let wrong = failure(
            self,
            Diagnostic::new(
                DiagnosticKind::CheckFailed,
                span,
                "this list took its memory from another allocator",
                WRONG_WHY,
            ),
            &[When::Text("it uses allocator ".into()), word(0), When::Text(", and this is allocator ".into()), word(1)],
        );
        let out_of_memory = match out_of_memory {
            OutOfMemoryHandling::Crash => checked::OutOfMemory::Crash(Box::new(self.out_of_memory_failure(span))),
            OutOfMemoryHandling::Ignore => checked::OutOfMemory::Ignore,
            OutOfMemoryHandling::Flag(local) => checked::OutOfMemory::Flag(local),
        };
        Ok(checked::Growth {
            allocator: Box::new(allocator),
            standard: footprint.size,
            align: footprint.align,
            grow,
            step,
            wrong: Box::new(wrong),
            out_of_memory,
        })
    }

    /// Running out of memory with nothing to handle it (D96, D324).
    fn out_of_memory_failure(&self, span: Span) -> checked::CheckFailure {
        failure(
            self,
            Diagnostic::new(
                DiagnosticKind::CheckFailed,
                span,
                "`out_of_memory` failed, and nothing handled it",
                UNHANDLED_WHY,
            ),
            &[
                When::Text("the arena has room for ".into()),
                When::Operand(0),
                When::Text(" bytes, ".into()),
                When::Operand(1),
                When::Text(" are taken, and this needs ".into()),
                When::Operand(2),
                When::Text(" from byte ".into()),
                When::Operand(3),
            ],
        )
    }

    /// `os::heap()`, `std::mem::arena.new(room: 1 mib, alloc: Lend $heap)` and `std::coll::list.from([…], alloc: …)`,
    /// or `None` for any other call.
    pub fn memory_call(
        &mut self,
        body: &mut Body,
        expression: &syntax::Expression,
        name: &str,
        arguments: &[syntax::Argument],
        expected: Option<&Type>,
    ) -> Option<Check<super::expressions::Typed>> {
        use super::expressions::Typed;
        match name {
            "os::heap" => Some((|| {
                if let Some(argument) = arguments.first() {
                    return Err(self.not_yet(argument.span, "an argument to `os::heap()`".to_string()));
                }
                Ok(Typed::new(Type::Allocator(AllocatorKind::Heap), Kind::Heap))
            })()),
            "std::mem::arena.new" => Some(self.arena_new(body, expression, arguments)),
            "str.from" => Some(self.text_from(body, expression, arguments)),
            "std::coll::list.from" => Some(self.list_from(body, expression, arguments, expected)),
            _ => None,
        }
    }

    /// `str.from($n, alloc: Lend $heap)`: heap text from a number or other text (D326, D328, D331, D369).
    fn text_from(
        &mut self,
        body: &mut Body,
        expression: &syntax::Expression,
        arguments: &[syntax::Argument],
    ) -> Check<super::expressions::Typed> {
        let [value, allocator] = arguments else {
            return Err(self.not_yet(expression.span, "`str.from` with these arguments".to_string()));
        };
        let typed = self.expression(body, &value.value, None)?;
        let (_, kind) = self.allocator_argument(body, allocator)?;
        if kind != AllocatorKind::Heap {
            return Err(self.not_yet(allocator.span, "heap text from an arena, whose sizes aren't decided".to_string()));
        }
        let text = match &typed.value_type {
            Type::Str(_) => Kind::TextCopy(Box::new(typed.value)),
            Type::Integer { .. }
            | Type::Float { kind: super::types::FloatKind::Stable | super::types::FloatKind::Ieee, .. } => {
                Kind::TextOf(Box::new(typed.value))
            }
            Type::Unknown => return Err(Reported),
            other => {
                return Err(self.not_yet(value.span, format!("`str.from` on `{other}`")));
            }
        };
        Ok(super::expressions::Typed::new(Type::Str(StrSize::Growing), text))
    }

    fn arena_new(
        &mut self,
        body: &mut Body,
        expression: &syntax::Expression,
        arguments: &[syntax::Argument],
    ) -> Check<super::expressions::Typed> {
        let [room, allocator] = arguments else {
            return Err(self.not_yet(expression.span, "`std::mem::arena.new` with these arguments".to_string()));
        };
        let label = room.label.as_ref().map(|label| label.text.as_str());
        if label == Some("size") {
            if let Some(label) = &room.label {
                // trace: D386
                return Err(self.error(Diagnostic::refused_fix(
                    label.span,
                    "room",
                    "an arena's own bookkeeping sits outside it, so `room:` is exactly the space for lists (D386)",
                )));
            }
        }
        if label != Some("room") {
            return Err(self.not_yet(room.span, "an arena without `room:`".to_string()));
        }
        let bytes = self.byte_size(&room.value)?;
        let (_, kind) = self.allocator_argument(body, allocator)?;
        if kind != AllocatorKind::Heap {
            return Err(self.not_yet(allocator.span, "an arena inside another allocator".to_string()));
        }
        Ok(super::expressions::Typed::new(Type::Allocator(AllocatorKind::Arena), Kind::ArenaNew { room: bytes }))
    }

    /// A size in bytes, like `64 bytes`, `4 kib` or `1 mib` (D370).
    pub fn byte_size(&mut self, value: &syntax::Expression) -> Check<u64> {
        if let ExpressionKind::WithUnit { value: number, unit } = &value.kind
            && let Some(each) = unit.first().and_then(|first| super::bytes_in(&first.name.text))
            && unit.segments.len() == 1
            && let ExpressionKind::Integer(text) = &number.kind
            && let Ok(count) = text.replace('_', "").parse::<u64>()
        {
            return count.checked_mul(each).ok_or_else(|| self.not_yet(value.span, "this size".to_string()));
        }
        if let ExpressionKind::Integer(text) = &value.kind {
            return Err(self.error(Diagnostic::refused_fix(
                value.span,
                format!("{text} bytes"),
                "the language asks for units on things that have them, like sizes (D101, D370)",
            )));
        }
        Err(self.not_yet(value.span, "this size".to_string()))
    }

    /// `std::coll::list.from([1, 2, 3], alloc: Lend $heap)`: a growable list filled from a literal (D369).
    fn list_from(
        &mut self,
        body: &mut Body,
        expression: &syntax::Expression,
        arguments: &[syntax::Argument],
        expected: Option<&Type>,
    ) -> Check<super::expressions::Typed> {
        let [items, allocator] = arguments else {
            return Err(self.not_yet(expression.span, "`std::coll::list.from` with these arguments".to_string()));
        };
        let found = expected.cloned().and_then(|list_type| match &list_type {
            Type::List { item, max: None, grow, .. } => Some(((**item).clone(), *grow, list_type)),
            _ => None,
        });
        let Some((item, grow, list_type)) = found else {
            // trace: D369
            let callee = match &expression.kind {
                ExpressionKind::Call { callee, .. } => callee.span,
                _ => expression.span,
            };
            return Err(self.error(Diagnostic::refused(
                callee,
                "`std::coll::list.from` makes a list that grows on the heap, so it goes where one is written, with `len.max: inf items`",
                "a list that grows on the heap is filled with `std::coll::list.from([1, 2, 3], alloc: Lend $heap)`, since every allocation names its allocator (D321, D369)",
            )));
        };
        let ExpressionKind::List(written) = &items.value.kind else {
            return Err(self.not_yet(items.span, "`std::coll::list.from` without a `[ ]` literal".to_string()));
        };
        let mut values = Vec::new();
        for each in written {
            values.push(self.value_into(body, each, &item)?);
        }
        let allocator = self.allocator_argument(body, allocator)?;
        let growth = self.growth(&item, grow, allocator, expression.span, OutOfMemoryHandling::Crash)?;
        Ok(super::expressions::Typed::new(list_type, Kind::ListFrom { items: values, growth }))
    }

    /// `std::coll::list.done(Give $xs, alloc: Lend $heap)` and `std::mem::arena.done(Give $arena, alloc: Lend $heap)`
    /// as statements, or `None` for anything else (D323, D330, D331).
    pub fn memory_statement(
        &mut self,
        body: &mut Body,
        expression: &syntax::Expression,
    ) -> Option<Check<checked::Statement>> {
        let ExpressionKind::Call { callee, arguments, .. } = &expression.kind else { return None };
        let ExpressionKind::Path(path) = &callee.kind else { return None };
        let name = self.resolve_path(path);
        let arena = match name.as_str() {
            "std::coll::list.done" => false,
            "std::mem::arena.done" => true,
            "str.done" => return Some(self.text_done(body, expression, &name, arguments)),
            "std::coll::map.done" => return Some(self.map_done(body, expression, path.span, arguments)),
            _ => return None,
        };
        Some((|| {
            self.require_import(&name, path.span)?;
            let [owner, allocator] = arguments.as_slice() else {
                return Err(self.not_yet(expression.span, format!("`{name}` with these arguments")));
            };
            if owner.ownership.map(|(word, _)| word) != Some(Ownership::Give) {
                return Err(self.not_yet(owner.span, format!("`{name}` without `Give`")));
            }
            let Some(root) = root_name(&owner.value).cloned() else {
                return Err(self.not_yet(owner.span, format!("`{name}` on a fresh value")));
            };
            // One whose `let` was refused never got a value to end (D203), and one whose type was refused says
            // nothing more.
            if let Some(index) = body.find(&root.text)
                && body.variables[index].value_type.is_unknown()
            {
                return Err(Reported);
            }
            if let Some(index) = body.find(&root.text)
                && body.states[index] != super::State::Assigned
                && body.variables[index].origin == super::Origin::RefusedLet
            {
                return Err(self.error(Diagnostic::refused(
                    expression.span,
                    format!("`{}` never got a value", root.text),
                    "a variable is read only after it has one (D203)",
                )));
            }
            let Some(reached) = self.reach(body, &owner.value)? else { return Err(Reported) };
            let (allocator, _) = self.allocator_argument(body, allocator)?;
            match (&reached.value_type, arena) {
                (Type::List { max: None, .. }, false) | (Type::Allocator(AllocatorKind::Arena), true) => {}
                (Type::Unknown, _) => return Err(Reported),
                (other, _) => {
                    return Err(self.error(Diagnostic::refused(
                        owner.value.span,
                        format!(
                            "this is `{other}`, and `{name}` ends {}",
                            if arena { "an arena" } else { "a growable list" }
                        ),
                        "every `must_Give` owner ends with its own `done` (D331)",
                    )));
                }
            }
            self.give_away(body, &root, &owner.value)?;
            let local = reached.root.local;
            if arena {
                let alive = failure(
                    self,
                    Diagnostic::new(
                        DiagnosticKind::CheckFailed,
                        expression.span,
                        "this arena ends while lists from it are still alive",
                        "an arena counts what's still alive from it, and ending it then is a failed check, so nothing can dangle into a freed arena (D330)",
                    ),
                    &[When::Operand(0), When::Text(" still alive".into())],
                );
                let machine_type = reached.value_type.machine();
                return Ok(checked::Statement::ArenaDone {
                    arena: checked::Expression {
                        kind: Kind::Load(checked::Place { local, steps: reached.steps }),
                        machine_type,
                    },
                    alive: Box::new(alive),
                });
            }
            let wrong = failure(
                self,
                Diagnostic::new(
                    DiagnosticKind::CheckFailed,
                    expression.span,
                    "this list took its memory from another allocator",
                    WRONG_WHY,
                ),
                &[
                    When::Text("it uses allocator ".into()),
                    When::Operand(0),
                    When::Text(", and this is allocator ".into()),
                    When::Operand(1),
                ],
            );
            Ok(checked::Statement::ListDone {
                place: checked::Place { local, steps: reached.steps },
                allocator,
                wrong: Box::new(wrong),
            })
        })())
    }

    /// `str.done(Give $log, alloc: Lend $heap)`: ends heap text (D326, D331).
    fn text_done(
        &mut self,
        body: &mut Body,
        expression: &syntax::Expression,
        name: &str,
        arguments: &[syntax::Argument],
    ) -> Check<checked::Statement> {
        let [owner, allocator] = arguments else {
            return Err(self.not_yet(expression.span, format!("`{name}` with these arguments")));
        };
        if owner.ownership.map(|(word, _)| word) != Some(Ownership::Give) {
            return Err(self.not_yet(owner.span, format!("`{name}` without `Give`")));
        }
        let Some(root) = root_name(&owner.value).cloned() else {
            return Err(self.not_yet(owner.span, format!("`{name}` on a fresh value")));
        };
        let typed = self.expression(body, &owner.value, None)?;
        self.allocator_argument(body, allocator)?;
        if typed.value_type != Type::Str(StrSize::Growing) {
            return Err(self.error(Diagnostic::refused(
                owner.value.span,
                format!("this is `{}`, and `str.done` ends heap text", typed.value_type),
                "every `must_Give` owner ends with its own `done`, and only heap text owns memory to free (D326, D331)",
            )));
        }
        self.give_away(body, &root, &owner.value)?;
        Ok(checked::Statement::TextDone(typed.value))
    }

    /// Refuses giving away a value a `defer` will give away (D325).
    pub fn not_promised(&mut self, body: &Body, local: checked::LocalId, root: &syntax::Name) -> Check<()> {
        if let Some(promise) = body.promised.iter().find(|(each, _)| *each == local).map(|(_, span)| *span)
            && !body.running_defers
        {
            // trace: D325
            return Err(self.error(
                Diagnostic::refused(
                    root.span,
                    format!("`{}` is promised to a `defer`, so it can't be given away here", root.text),
                    "a deferred `Give` ends the value's duty when its block ends, so the value is yours to read and change until then, but not to give away (D325)",
                )
                .with_because(promise, "it's promised here".to_string()),
            ));
        }
        Ok(())
    }

    /// Records that a value is given away, ending its duty (D2, D325).
    pub fn give_away(&mut self, body: &mut Body, root: &syntax::Name, value: &syntax::Expression) -> Check<()> {
        if let Some((local, path)) = self.any_place(body, value) {
            self.not_promised(body, local, root)?;
            self.only_lent(body, local, root, &path, value.span)?;
            body.take_read(local);
            let site = body.site(root.span);
            let shown = self.slice(value.span).to_string();
            body.events.push(crate::flow::Event::Give {
                place: crate::flow::Place { local, path, shown },
                spot: crate::flow::Spot { variable: root.span, place: value.span, site, marker: site },
            });
        }
        Ok(())
    }

    /// Whether appending to this list needs an allocator, and checks the one given (D321, D322).
    pub fn append_growth(
        &mut self,
        body: &mut Body,
        list: &Type,
        call: &syntax::Expression,
        allocator: Option<&syntax::Argument>,
        out_of_memory: OutOfMemoryHandling,
    ) -> Check<Option<checked::Growth>> {
        let Type::List { item, max, grow, .. } = list else { return Ok(None) };
        match (max, allocator) {
            (Some(_), Some(argument)) => Err(self.error(Diagnostic::refused(
                argument.ownership.map_or(argument.value.span, |(_, word)| word.to(argument.value.span)),
                "a list with a finite `len.max` holds its items inside it, so it takes no allocator",
                "a list with a finite `len.max` needs no allocator, since its size is known (D321)",
            ))),
            (Some(_), None) => Ok(None),
            (None, None) => Err(self.error(Diagnostic::refused(
                call.span,
                "this list grows on the heap, so `append` says where the memory comes from",
                ALLOCATOR_WHY,
            ))),
            (None, Some(argument)) => {
                let allocator = self.allocator_argument(body, argument)?;
                self.growth(item, *grow, allocator, call.span, out_of_memory).map(Some)
            }
        }
    }
}

/// What an `append` that runs out of memory does, before it's turned into checked code.
pub enum OutOfMemoryHandling {
    Crash,
    Ignore,
    Flag(checked::LocalId),
}

fn round_up(value: u64, align: u64) -> Option<u64> {
    let align = align.max(1);
    value.checked_add(align - 1).map(|value| value / align * align)
}

/// A struct's standard size from its fields' (D319, D388): natural padding in the written order, sorted by
/// alignment for `.compact`, and no padding for `.packed`. `.any_order` is the compiler's own choice, which is
/// `.compact`'s.
fn struct_footprint(fields: &[Footprint], layout: super::aggregates::Layout) -> Option<Footprint> {
    use super::aggregates::Layout;
    let mut order: Vec<Footprint> = fields.to_vec();
    if matches!(layout, Layout::Compact | Layout::AnyOrder) {
        // A stable sort keeps ties in the written order (D388).
        order.sort_by(|a, b| b.align.cmp(&a.align));
    }
    let packed = layout == Layout::Packed;
    let mut size = 0u64;
    let mut align = 1u64;
    let mut per_target = false;
    for field in &order {
        let field_align = if packed { 1 } else { field.align };
        size = round_up(size, field_align)?.checked_add(field.size)?;
        align = align.max(field_align);
        per_target |= field.per_target;
    }
    Some(Footprint { size: round_up(size, align)?, align, per_target })
}

impl Checker<'_> {
    /// `std::coll::map.done(Give $m, alloc: Lend $heap)`: ends a map, which owns its memory (D325, D331). A map is
    /// only ever `empty` so far, since its operations aren't decided, so there's nothing yet to free.
    fn map_done(
        &mut self,
        body: &mut Body,
        expression: &syntax::Expression,
        path_span: crate::source::Span,
        arguments: &[syntax::Argument],
    ) -> Check<checked::Statement> {
        self.require_import("std::coll::map.done", path_span)?;
        let [owner, allocator] = arguments else {
            return Err(self.not_yet(expression.span, "`std::coll::map.done` with these arguments".to_string()));
        };
        let Some(root) = root_name(&owner.value).cloned() else {
            return Err(self.not_yet(owner.span, "`std::coll::map.done` on a fresh value".to_string()));
        };
        let Some(reached) = self.reach(body, &owner.value)? else { return Err(Reported) };
        match &reached.value_type {
            Type::Map { .. } => {}
            Type::Unknown => return Err(Reported),
            other => {
                return Err(self.error(Diagnostic::refused(
                    owner.value.span,
                    format!("this is `{other}`, and `std::coll::map.done` ends a map"),
                    "every `must_Give` owner ends with its own `done` (D331)",
                )));
            }
        }
        self.allocator_argument(body, allocator)?;
        // trace: D325, D331
        self.give_away(body, &root, &owner.value)?;
        Ok(checked::Statement::Evaluate(checked::Expression {
            kind: Kind::Integer(0),
            machine_type: checked::MachineType::I64,
        }))
    }
}
