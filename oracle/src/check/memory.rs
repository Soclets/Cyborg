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
use crate::report::{Outcome, Report, Span};
use crate::types::Type;

/// Whether a type is an allocator the oracle has. (D332)
pub(super) fn is_allocator(value_type: &Type) -> bool {
    matches!(value_type, Type::Struct(name) if name == "os::heap")
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
        };
        let (items_node, _) = self.check_expression(&items.value, Some(&holder))?;
        let mut handed = Vec::new();
        let allocator = self.allocator_argument(&allocator.value, &mut handed)?;
        Ok(RawCall {
            node: Node {
                span,
                kind: NodeKind::ListFrom {
                    items: Box::new(items_node),
                    allocator: Box::new(allocator),
                },
            },
            value_type: Type::List {
                item,
                min,
                max: None,
                len_type,
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
