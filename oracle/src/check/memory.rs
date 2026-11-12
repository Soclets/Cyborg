//! Explicit allocation: allocators, lists that grow on the heap, and ending them with `done`.
//! (D321, D322, D323, D325, D331, D332)
//!
//! An allocator is a built-in struct type, `os::heap`, made with `os::heap()`. A list with
//! `len.max: inf items` doesn't hold its allocator: every call that allocates is handed one, and
//! the list remembers which, by number, so handing it another is a failed check. Such a list is
//! `must_Give`, and `std::list.done` ends it.

use super::expression::RawCall;
use super::hand::HandedOff;
use super::{Checker, Failure, ParameterInfo};
use crate::ast::{Argument, Expression, ExpressionKind, ParameterWord};
use crate::ir::{self, Node, NodeKind};
use crate::report::{Outcome, Report, ReportExtra, Span};
use crate::types::Type;

/// Whether a type is an allocator the oracle has: the OS heap, or an arena. (D330, D332)
pub(super) fn is_allocator(value_type: &Type) -> bool {
    matches!(value_type, Type::Struct(name) if name == "os::heap") || is_arena(value_type)
}

/// An arena, which owns its block, so it's `must_Give` and ends with `arena.done`. (D330)
pub(super) fn is_arena(value_type: &Type) -> bool {
    matches!(value_type, Type::Struct(name) if name == "std::mem::arena")
}

/// A list that can grow on the heap, which takes an allocator and is `must_Give`. (D323)
pub(super) fn is_growable(value_type: &Type) -> bool {
    matches!(value_type, Type::List { max: None, .. })
}

/// What `std::list.done` or `std::map.done` ends: a growable list, or any map, since a map
/// always owns heap memory. (D323, D325)
fn ended_by(done: &str, value_type: &Type) -> bool {
    match done {
        "map.done" => matches!(value_type, Type::Map { .. }),
        _ => is_growable(value_type),
    }
}

/// A literal size: a number of `bytes`, `kib` or `mib`, in bytes. (D370)
/// trace: D370
pub(super) fn byte_size(value: &Expression) -> Option<i128> {
    let ExpressionKind::WithUnit { number, unit } = &value.kind else {
        return None;
    };
    let scale = match unit.text.as_str() {
        "bytes" => 1,
        "kib" => 1024,
        "mib" => 1024 * 1024,
        _ => return None,
    };
    super::expression::literal_integer(number)
        .filter(|count| *count > 0)
        .and_then(|count| count.checked_mul(scale))
}

/// Running out of memory is a failure, handled like any other. (D321, D324)
pub(super) fn out_of_memory() -> Failure {
    Failure {
        short: "out_of_memory".to_string(),
        full: "out_of_memory".to_string(),
        data: Vec::new(),
    }
}

const WHY_ALLOCATOR: &str = "anything that takes heap memory is handed an allocator, as in `alloc: Lend $heap`, so where memory comes from is always said (D321, D322)";

impl Checker<'_> {
    /// The name of an allocator in scope, for a fix, or `$heap` when there's none yet.
    pub(super) fn an_allocator_name(&self) -> String {
        self.context
            .scopes
            .iter()
            .rev()
            .flat_map(|scope| scope.iter())
            .filter(|(_, slot)| {
                self.context
                    .variables
                    .get(**slot)
                    .is_some_and(|variable| is_allocator(&variable.held_type))
            })
            .map(|(name, _)| format!("${name}"))
            .min()
            .unwrap_or_else(|| "$heap".to_string())
    }

    /// The type of the variable an argument names, through any ownership word, read without
    /// handing it over yet.
    pub(super) fn variable_type_of(&self, value: &Expression) -> Option<Type> {
        let inner = match &value.kind {
            ExpressionKind::Hand { value, .. } => value,
            _ => value,
        };
        let ExpressionKind::Variable(name) = &inner.kind else {
            return None;
        };
        let slot = self.find_variable(name)?;
        let found = &self.context.variables.get(slot)?.held_type;
        match found {
            Type::Borrow { inner, .. } => Some((**inner).clone()),
            other => Some(other.clone()),
        }
    }

    /// `os::heap()`, the operating system's allocator. (D332)
    /// trace: D332
    pub(super) fn check_heap_call(arguments: &[Argument], span: Span) -> Outcome<RawCall> {
        if let Some(extra) = arguments.first() {
            return Err(Report::refused(
                extra.value.span,
                "`os::heap()` takes nothing",
                "`os::heap()` gives the allocator that takes memory from the operating system (D332)",
            ));
        }
        Ok(RawCall {
            node: Node {
                span,
                kind: NodeKind::Allocator,
            },
            value_type: Type::Struct("os::heap".to_string()),
            failures: Vec::new(),
            unnamed: None,
        })
    }

    /// An `alloc:` argument, lent: the allocator a call takes memory from. (D322)
    /// trace: D322
    pub(super) fn allocator_argument(
        &mut self,
        value: &Expression,
        handed: &mut Vec<HandedOff>,
    ) -> Outcome<Node> {
        let Some(allocator_type) = self.variable_type_of(value).filter(is_allocator) else {
            return Err(Report::refused(
                value.span,
                "`alloc:` takes an allocator, like `Lend $heap`",
                WHY_ALLOCATOR,
            ));
        };
        let parameter = ParameterInfo {
            name: "alloc".to_string(),
            label: Some("alloc".to_string()),
            word: ParameterWord::Bare,
            parameter_type: allocator_type,
            default: None,
            label_has_unit: false,
        };
        Ok(match self.check_argument(value, &parameter, handed)? {
            ir::Argument::Value(node) => node,
            ir::Argument::Alias(place) => Node {
                span: value.span,
                kind: NodeKind::Read(place),
            },
        })
    }

    /// Splits an `append`'s arguments into its `val:` and its `alloc:`. A growable list needs
    /// one, and a list with a finite `len.max` needs none. (D321, D322)
    /// trace: D321, D322
    pub(super) fn split_allocator(
        arguments: &[Argument],
        growable: bool,
        span: Span,
    ) -> Outcome<(Vec<Argument>, Option<&Argument>)> {
        let allocator = arguments.iter().find(|argument| {
            argument
                .label
                .as_ref()
                .is_some_and(|label| label.text == "alloc")
        });
        let rest: Vec<Argument> = arguments
            .iter()
            .filter(|argument| {
                argument
                    .label
                    .as_ref()
                    .is_none_or(|label| label.text != "alloc")
            })
            .cloned()
            .collect();
        match (growable, allocator) {
            (true, None) => Err(Report::refused(
                span,
                "this list grows on the heap, so `append` says where the memory comes from",
                WHY_ALLOCATOR,
            )),
            (false, Some(allocator)) => Err(Report::refused(
                allocator.value.span,
                "a list with a finite `len.max` holds its items inside it, so it takes no allocator",
                "a list with a finite `len.max` needs no allocator, since its size is known (D321)",
            )),
            _ => Ok((rest, allocator)),
        }
    }

    /// `std::list.done(Give $xs, alloc: Lend $heap)` ends a growable list, freeing it with the
    /// allocator it came from, which is checked. (D323, D331)
    /// trace: D323, D331
    pub(super) fn check_list_done(
        &mut self,
        done: &str,
        arguments: &[Argument],
        span: Span,
        path_span: Span,
    ) -> Outcome<RawCall> {
        const WHY: &str = "`std::coll::list.done(Give $xs, alloc: Lend $heap);` ends a list that grows on the heap, and `std::coll::map.done` a map, freeing it with the allocator it came from (D323, D325, D331)";
        let (list, allocator) = match arguments {
            [list, allocator]
                if list.label.is_none()
                    && allocator
                        .label
                        .as_ref()
                        .is_some_and(|label| label.text == "alloc") =>
            {
                (list, allocator)
            }
            _ => {
                return Err(Report::refused(
                    path_span,
                    format!(
                        "`std::{}` takes what it ends, then `alloc:`",
                        super::grouped(done)
                    ),
                    WHY,
                ))
            }
        };
        let Some(list_type) = self
            .variable_type_of(&list.value)
            .filter(|found| ended_by(done, found))
        else {
            return Err(Report::refused(
                list.value.span,
                if done == "map.done" {
                    "`std::coll::map.done` ends a map"
                } else {
                    "`std::coll::list.done` ends a list that grows on the heap, with `len.max: inf items`"
                },
                WHY,
            ));
        };
        let parameter = ParameterInfo {
            name: "list".to_string(),
            label: None,
            word: ParameterWord::Given,
            parameter_type: list_type,
            default: None,
            label_has_unit: false,
        };
        let mut handed = Vec::new();
        let list_node = match self.check_argument(&list.value, &parameter, &mut handed)? {
            ir::Argument::Value(node) => node,
            ir::Argument::Alias(place) => Node {
                span: list.value.span,
                kind: NodeKind::Read(place),
            },
        };
        let allocator_node = self.allocator_argument(&allocator.value, &mut handed)?;
        Ok(RawCall {
            node: Node {
                span,
                kind: NodeKind::ListDone {
                    list: Box::new(list_node),
                    allocator: Box::new(allocator_node),
                },
            },
            value_type: Type::Void,
            failures: Vec::new(),
            unnamed: None,
        })
    }
}

impl Checker<'_> {
    /// `std::mem::arena.new(room: 1 mib, alloc: Lend $heap)` takes one block from another
    /// allocator, which `std::mem::arena.done(Give $arena, alloc: Lend $heap)` hands back. (D330,
    /// D333, D378)
    /// trace: D330, D333
    #[allow(clippy::too_many_lines)] // `new` and `done` share their argument checks.
    pub(super) fn check_arena_call(
        &mut self,
        word: &str,
        arguments: &[Argument],
        span: Span,
        path_span: Span,
    ) -> Outcome<RawCall> {
        const WHY: &str = "an arena takes one block, `std::mem::arena.new(room: 1 mib, alloc: Lend $heap)`, hands it out, and gives it back all at once with `std::mem::arena.done(Give $arena, alloc: Lend $heap)` (D330, D333)";
        let [first, allocator] = arguments else {
            return Err(Report::refused(
                path_span,
                format!("`std::mem::arena.{word}` takes two arguments"),
                WHY,
            ));
        };
        if allocator
            .label
            .as_ref()
            .is_none_or(|label| label.text != "alloc")
        {
            return Err(Report::refused(
                allocator.value.span,
                format!("`std::mem::arena.{word}` takes `alloc:` last"),
                WHY,
            ));
        }
        let mut handed = Vec::new();
        match word {
            "new" => {
                // trace: D386
                if let Some(label) = first.label.as_ref().filter(|label| label.text == "size") {
                    return Err(Report::refused(
                        label.span,
                        "an arena says `room:`, the space for what's put in it",
                        "an arena's own bookkeeping sits outside it, so `room:` is exactly the space for lists (D386)",
                    )
                    .fix("room"));
                }
                if first
                    .label
                    .as_ref()
                    .is_none_or(|label| label.text != "room")
                {
                    return Err(Report::refused(
                        first.value.span,
                        "`std::mem::arena.new` takes `room:`",
                        WHY,
                    ));
                }
                let Some(bytes) = byte_size(&first.value) else {
                    return Err(Report::not_designed(
                        first.value.span,
                        "an arena's `room:` that isn't a literal number of `bytes`, `kib` or `mib`",
                        "",
                    ));
                };
                let allocator = self.allocator_argument(&allocator.value, &mut handed)?;
                Ok(RawCall {
                    node: Node {
                        span,
                        kind: NodeKind::ArenaNew {
                            bytes,
                            allocator: Box::new(allocator),
                        },
                    },
                    value_type: Type::Struct("std::mem::arena".to_string()),
                    failures: vec![out_of_memory()],
                    unnamed: None,
                })
            }
            "done" => {
                if first.label.is_some()
                    || !self
                        .variable_type_of(&first.value)
                        .is_some_and(|found| is_arena(&found))
                {
                    return Err(Report::refused(
                        first.value.span,
                        "`std::mem::arena.done` ends an arena",
                        WHY,
                    ));
                }
                let parameter = ParameterInfo {
                    name: "arena".to_string(),
                    label: None,
                    word: ParameterWord::Given,
                    parameter_type: Type::Struct("std::mem::arena".to_string()),
                    default: None,
                    label_has_unit: false,
                };
                let arena = match self.check_argument(&first.value, &parameter, &mut handed)? {
                    ir::Argument::Value(node) => node,
                    ir::Argument::Alias(place) => Node {
                        span: first.value.span,
                        kind: NodeKind::Read(place),
                    },
                };
                let allocator = self.allocator_argument(&allocator.value, &mut handed)?;
                Ok(RawCall {
                    node: Node {
                        span,
                        kind: NodeKind::ArenaDone {
                            arena: Box::new(arena),
                            allocator: Box::new(allocator),
                        },
                    },
                    value_type: Type::Void,
                    failures: Vec::new(),
                    unnamed: None,
                })
            }
            other => Err(Report::not_designed(
                path_span,
                format!("`std::mem::arena.{other}`"),
                "",
            )),
        }
    }

    /// `std::coll::list.from([1, 2, 3], alloc: Lend $heap)` fills a growable list from items,
    /// since filling one allocates, and every allocation names its allocator. It takes its type
    /// from where it goes, as a literal does. (D321, D369)
    /// trace: D369
    pub(super) fn check_list_from(
        &mut self,
        arguments: &[Argument],
        span: Span,
        path_span: Span,
    ) -> Outcome<RawCall> {
        const WHY: &str = "a list that grows on the heap is filled with `std::coll::list.from([1, 2, 3], alloc: Lend $heap)`, since every allocation names its allocator (D321, D369)";
        let target = self.context.call_expected.clone();
        let Some(Type::List {
            item,
            min,
            max: None,
            len_type,
            grow,
        }) = target.clone()
        else {
            return Err(Report::refused(
                path_span,
                "`std::coll::list.from` makes a list that grows on the heap, so it goes where one is written, with `len.max: inf items`",
                WHY,
            ));
        };
        let [items, allocator] = arguments else {
            return Err(Report::refused(
                path_span,
                "`std::coll::list.from` takes the items, then `alloc:`",
                WHY,
            ));
        };
        if items.label.is_some()
            || allocator
                .label
                .as_ref()
                .is_none_or(|label| label.text != "alloc")
        {
            return Err(Report::refused(
                path_span,
                "`std::coll::list.from` takes the items, then `alloc:`",
                WHY,
            ));
        }
        let ExpressionKind::Items(written) = &items.value.kind else {
            return Err(Report::not_designed(
                items.value.span,
                "`std::coll::list.from` on anything but `[ ]` items",
                "",
            ));
        };
        // The items go in as a list just big enough for them, checked against `len.min:`.
        let count = i128::try_from(written.len()).unwrap_or(i128::MAX);
        let holder = Type::List {
            item: item.clone(),
            min,
            max: Some(count),
            len_type,
            grow: grow.clone(),
        };
        let (items_node, _) = self.check_expression(&items.value, Some(&holder))?;
        let mut handed = Vec::new();
        let allocator = self.allocator_argument(&allocator.value, &mut handed)?;
        let growing = Box::new(ir::Growing {
            item: self.item_size(&item),
            grow: grow.clone(),
        });
        Ok(RawCall {
            node: Node {
                span,
                kind: NodeKind::ListFrom {
                    items: Box::new(items_node),
                    allocator: Box::new(allocator),
                    growing,
                },
            },
            value_type: Type::List {
                item,
                min,
                max: None,
                len_type,
                grow,
            },
            failures: vec![out_of_memory()],
            unnamed: None,
        })
    }

    /// `str.from($n, alloc: Lend $heap)` makes heap text, and `str.done(Give $log, alloc: Lend
    /// $heap)` ends it. They're led by the type's own name, since `str` is a base type. (D331)
    /// trace: D326, D328, D331
    pub(super) fn check_text_call(
        &mut self,
        word: &str,
        arguments: &[Argument],
        span: Span,
        path_span: Span,
    ) -> Outcome<RawCall> {
        let why = "heap text comes from `str.from(…, alloc: Lend $heap)`, and `str.done(Give $log, alloc: Lend $heap)` ends it (D326, D331)";
        let [value, allocator] = arguments else {
            return Err(Report::refused(
                path_span,
                format!("`str.{word}` takes a value, then `alloc:`"),
                why,
            ));
        };
        if value.label.is_some()
            || allocator
                .label
                .as_ref()
                .is_none_or(|label| label.text != "alloc")
        {
            return Err(Report::refused(
                path_span,
                format!("`str.{word}` takes a value, then `alloc:`"),
                why,
            ));
        }
        let mut handed = Vec::new();
        if word == "done" {
            if self.variable_type_of(&value.value) != Some(Type::GrowableStr) {
                return Err(Report::refused(
                    value.value.span,
                    "`str.done` ends a string that grows on the heap, with `len.max: inf bytes`",
                    why,
                ));
            }
            let parameter = ParameterInfo {
                name: "text".to_string(),
                label: None,
                word: ParameterWord::Given,
                parameter_type: Type::GrowableStr,
                default: None,
                label_has_unit: false,
            };
            let text = match self.check_argument(&value.value, &parameter, &mut handed)? {
                ir::Argument::Value(node) => node,
                ir::Argument::Alias(place) => Node {
                    span: value.value.span,
                    kind: NodeKind::Read(place),
                },
            };
            let allocator = self.allocator_argument(&allocator.value, &mut handed)?;
            return Ok(RawCall {
                node: Node {
                    span,
                    kind: NodeKind::TextDone {
                        text: Box::new(text),
                        allocator: Box::new(allocator),
                    },
                },
                value_type: Type::Void,
                failures: Vec::new(),
                unnamed: None,
            });
        }
        let (node, found) = self.check_expression(&value.value, None)?;
        let text = match &found {
            Type::Integer { .. } => Node {
                span,
                kind: NodeKind::IntegerToStr(Box::new(node)),
            },
            Type::Float { kind, rules, .. } => Node {
                span,
                kind: NodeKind::FloatToStr(crate::ir::Number::Float(*kind, *rules), Box::new(node)),
            },
            // `str.from` takes a string as well as a number, and copies its text. (D369)
            // trace: D369
            Type::Str | Type::BoundedStr(_) | Type::GrowableStr => node,
            other => {
                return Err(Report::not_built(
                    value.value.span,
                    format!("`str.from` on `{other}`"),
                ));
            }
        };
        let allocator = self.allocator_argument(&allocator.value, &mut handed)?;
        Ok(RawCall {
            node: Node {
                span,
                kind: NodeKind::TextFrom {
                    text: Box::new(text),
                    allocator: Box::new(allocator),
                },
            },
            value_type: Type::GrowableStr,
            failures: vec![out_of_memory()],
            unnamed: None,
        })
    }
}

/// What can't be sized: what to say, and whether it's the compiler's own choice rather than an
/// undecided question.
type NoSize = (String, bool);

impl Checker<'_> {
    /// What one item of this type takes in a list's block, for each pointer width a target
    /// can have. (D370, D387, D389)
    /// trace: D370, D387, D389
    pub(super) fn item_size(&self, item: &Type) -> ir::ItemSize {
        match (self.layout_of(item, 4), self.layout_of(item, 8)) {
            (Ok(four), Ok(eight)) if four == eight => ir::ItemSize::Fixed {
                size: four.0,
                align: four.1,
            },
            (Ok(four), Ok(eight)) => ir::ItemSize::ByPointer { four, eight },
            (Err((what, compiler_own)), _) | (_, Err((what, compiler_own))) => {
                ir::ItemSize::Unknown { what, compiler_own }
            }
        }
    }

    /// A type's size and alignment in bytes, where a pointer takes `pointer` bytes: numbers and
    /// `bool` their own size, an array its items, a bounded list its length then its items,
    /// and a struct by its layout. (D319, D370, D388, D389)
    fn layout_of(&self, value_type: &Type, pointer: u64) -> Result<(u64, u64), NoSize> {
        let undecided = |what: String| Err((what, false));
        match value_type {
            Type::Integer { kind, .. } => {
                let bytes = u64::from(kind.bits() / 8);
                Ok((bytes, bytes))
            }
            Type::Float { kind, .. } => Ok(match kind {
                crate::types::FloatKind::F32 => (4, 4),
                crate::types::FloatKind::F64 => (8, 8),
            }),
            Type::Bool => Ok((1, 1)),
            Type::Pointer { .. } => Ok((pointer, pointer)),
            // A view is where its text or items are, and how many, each pointer-sized. (D390)
            // trace: D390
            Type::Str | Type::ListView { .. } => Ok((pointer.saturating_mul(2), pointer)),
            Type::Array { item, len, .. } => {
                let (size, align) = self.layout_of(item, pointer)?;
                let count = u64::try_from(*len).unwrap_or(u64::MAX);
                Ok((size.saturating_mul(count), align))
            }
            Type::List {
                item,
                max: Some(max),
                len_type,
                ..
            } => {
                let (size, align) = self.layout_of(item, pointer)?;
                let length = u64::from(len_type.bits() / 8);
                let count = u64::try_from(*max).unwrap_or(u64::MAX);
                let items_at = round_up(length, align);
                let whole_align = align.max(length);
                Ok((
                    round_up(
                        items_at.saturating_add(size.saturating_mul(count)),
                        whole_align,
                    ),
                    whole_align,
                ))
            }
            Type::Struct(name) => {
                let Some(info) = self.struct_info(name) else {
                    return undecided(format!("how much `{name}` takes"));
                };
                let mut fields = Vec::new();
                for (_, field) in &info.fields {
                    fields.push(self.layout_of(field, pointer)?);
                }
                Ok(struct_layout(info.layout, fields))
            }
            other => undecided(format!("how much `{other}` takes in a list's block")),
        }
    }
}

/// `value` rounded up to a multiple of `step`.
fn round_up(value: u64, step: u64) -> u64 {
    if step == 0 {
        return value;
    }
    value.div_ceil(step).saturating_mul(step)
}

/// A struct's size and alignment from its fields', by its layout: the written order with
/// natural padding, sorted by alignment for `struct.compact`, and no padding for
/// `struct.packed`. `struct.any_order` is the compiler's own, and the oracle sorts it like
/// `struct.compact`. (D319, D388, D389)
fn struct_layout(layout: crate::ast::StructLayout, mut fields: Vec<(u64, u64)>) -> (u64, u64) {
    use crate::ast::StructLayout;
    match layout {
        StructLayout::Packed => {
            let size = fields.iter().map(|(size, _)| *size).sum();
            return (size, 1);
        }
        // Largest alignment first, ties in the written order, which a stable sort keeps.
        // `struct.any_order` is the compiler's own choice, and what depends on it is outside the
        // shared tests, so the oracle picks this one too. (D388, D389)
        // trace: D388, D389
        StructLayout::Compact | StructLayout::AnyOrder => {
            fields.sort_by_key(|field| std::cmp::Reverse(field.1));
        }
        StructLayout::Written => {}
    }
    let mut offset = 0;
    let mut align = 1;
    for (size, field_align) in fields {
        offset = round_up(offset, field_align).saturating_add(size);
        align = align.max(field_align);
    }
    (round_up(offset, align), align)
}
