//! `simd`: a base type holding several values, its lanes, worked on together. (D291 to D307)
//!
//! At run time a vector is a list of its lanes. An operator on two of them works on every lane,
//! and a lane follows its type's rules exactly, so a failure says which lane. Combining the
//! lanes uses a fixed pairwise order, so the answer never depends on the CPU.

use super::expression::RawCall;
use super::Checker;
use crate::ast::{Argument, CompareOperator, Describe, Expression, ExpressionKind, Fill};
use crate::ir::{Arithmetic, Node, NodeKind, Number, Reduction};
use crate::report::{Outcome, Report, ReportExtra, Span};
use crate::types::Type;

const WHY_SIMD: &str = "`simd<i32>[lanes: 4]` holds several values, its lanes, and an operator on two of them works on every lane (D291)";

/// The lane type and count of a `simd`.
pub(super) fn simd_parts(value_type: &Type) -> Option<(&Type, usize)> {
    match value_type {
        Type::Simd { item, lanes } => Some((item, *lanes)),
        _ => None,
    }
}

/// How a number lane is worked out, for the runner's maths.
fn lane_number(item: &Type) -> Option<Number> {
    match item {
        Type::Integer { kind, .. } => Some(Number::Integer(*kind)),
        Type::Float { kind, rules, .. } => Some(Number::Float(*kind, *rules)),
        _ => None,
    }
}

impl Checker<'_> {
    /// `simd<i32>[lanes: 4]`: one lane type, bare in `< >`, and its count. `lanes: max` is
    /// the most one instruction works on for the target being built, which the oracle doesn't
    /// have. (D291, D292)
    /// trace: D291, D292
    pub(super) fn resolve_simd_type(
        &self,
        fill: &[Fill],
        describe: &[Describe],
        span: Span,
    ) -> Outcome<Type> {
        let [Fill {
            label: None,
            written_type,
        }] = fill
        else {
            return Err(Report::refused(
                span,
                "`simd` fills in one lane type, bare, in `< >`",
                WHY_SIMD,
            ));
        };
        let item = self.resolve_type(written_type)?;
        match &item {
            Type::Integer { bound: None, .. } | Type::Float { bound: None, .. } | Type::Bool => {}
            Type::Integer { .. } | Type::Float { .. } | Type::Secret(_) => {
                return Err(Report::not_built(
                    written_type.span,
                    format!("a `simd` of `{item}`"),
                ));
            }
            _ => {
                return Err(Report::refused(
                    written_type.span,
                    format!("a `simd`'s lanes are numbers or `bool`, and this is `{item}`"),
                    WHY_SIMD,
                ));
            }
        }
        let [entry] = describe else {
            return Err(Report::refused(
                span,
                "a `simd` says its `lanes:`",
                WHY_SIMD,
            ));
        };
        if entry.label.text != "lanes" {
            return Err(Report::refused(
                entry.label.span,
                format!("a `simd` says `lanes:`, not `{}:`", entry.label.text),
                WHY_SIMD,
            ));
        }
        if let ExpressionKind::Path(path) = &entry.value.kind {
            if path.text == "max" {
                return Err(Report::not_built(
                    entry.value.span,
                    "`lanes: max`, which is the target's own width, and the oracle builds for no target",
                ));
            }
        }
        let lanes = super::expression::literal_integer(&entry.value)
            .and_then(|count| usize::try_from(count).ok())
            .filter(|count| *count >= 1)
            .ok_or_else(|| {
                Report::refused(
                    entry.value.span,
                    "`lanes:` is a whole number of lanes, or `max`",
                    "`lanes:` takes any count, or `max` (D292)",
                )
            })?;
        Ok(Type::Simd {
            item: Box::new(item),
            lanes,
        })
    }

    /// An operator on two `simd` values works on every lane, and gives a `simd` of the same
    /// count. (D291, D293)
    /// trace: D291, D293
    pub(super) fn check_simd_binary(
        &self,
        operator: Arithmetic,
        (left_node, left_type): (Node, Type),
        (right_node, right_type): (Node, Type),
        right: &Expression,
        span: Span,
    ) -> Outcome<(Node, Type)> {
        let (Some((left_item, lanes)), Some((right_item, right_lanes))) =
            (simd_parts(&left_type), simd_parts(&right_type))
        else {
            return Err(Report::not_designed(
                span,
                format!(
                    "an operator between `{left_type}` and `{right_type}`, a `simd` and one value"
                ),
                "12.2",
            ));
        };
        if lanes != right_lanes || left_item != right_item {
            return Err(Report::refused(
                span,
                format!("this mixes `{left_type}` and `{right_type}`"),
                "an operator on two `simd` values works lane by lane, so they have the same lane type and count (D291)",
            ));
        }
        let (number, result_item) =
            self.arithmetic_type(operator, left_item, right_item, right, span)?;
        let operator = match operator {
            Arithmetic::Saturate(word, ..) => {
                let (low, high, _) = Self::saturate_bounds(left_item, right_item, span)?;
                Arithmetic::Saturate(word, low, high)
            }
            other => other,
        };
        Ok((
            Node {
                span,
                kind: NodeKind::Lanewise {
                    operator,
                    number,
                    left: Box::new(left_node),
                    right: Box::new(right_node),
                },
            },
            Type::Simd {
                item: Box::new(result_item),
                lanes,
            },
        ))
    }

    /// Comparing two `simd` values gives a `simd<bool>`, one answer per lane, so the code
    /// chooses "every lane" or "any lane". (D301)
    /// trace: D301
    pub(super) fn simd_compare(
        operator: CompareOperator,
        (left_node, left_type): (Node, Type),
        (right_node, right_type): (Node, Type),
        span: Span,
    ) -> Outcome<(Node, Type)> {
        let (Some((left_item, lanes)), Some((right_item, right_lanes))) =
            (simd_parts(&left_type), simd_parts(&right_type))
        else {
            return Err(Report::not_designed(
                span,
                format!("comparing `{left_type}` and `{right_type}`, a `simd` and one value"),
                "12.2",
            ));
        };
        if lanes != right_lanes || left_item != right_item {
            return Err(Report::refused(
                span,
                format!("this compares `{left_type}` and `{right_type}`"),
                "comparing two `simd` values works lane by lane, so they have the same lane type and count (D301)",
            ));
        }
        Ok((
            Node {
                span,
                kind: NodeKind::LanewiseCompare {
                    operator,
                    left: Box::new(left_node),
                    right: Box::new(right_node),
                },
            },
            Type::Simd {
                item: Box::new(Type::Bool),
                lanes,
            },
        ))
    }

    /// `if` needs one `bool`, so it says whether every lane or any lane. (D301)
    /// trace: D301
    pub(super) fn refuse_simd_condition(found: &Type, condition: &Expression) -> Outcome<()> {
        if let Some((Type::Bool, _)) = simd_parts(found) {
            return Err(Report::refused(
                condition.span,
                format!("this is `{found}`, one answer per lane, where `if` needs one `bool`"),
                "`if` takes one `bool`, so the code says `.%all_true()` for every lane or `.%any_true()` for any lane, rather than one being picked quietly (D301)",
            ));
        }
        Ok(())
    }

    /// A `simd`'s methods: combining its lanes, choosing lane by lane, and reordering. (D297,
    /// D302, D304 to D307)
    /// trace: D297, D302, D304, D305, D306, D307
    pub(super) fn check_simd_method(
        &mut self,
        receiver: Node,
        receiver_type: &Type,
        name: &crate::ast::Name,
        arguments: &[Argument],
        span: Span,
    ) -> Outcome<RawCall> {
        let (base, narrowing) = match name.text.split_once('.') {
            Some((base, narrowing)) => (base, Some(narrowing)),
            None => (name.text.as_str(), None),
        };
        let Some((item, lanes)) = simd_parts(receiver_type) else {
            return Err(Report::refused(span, "this isn't a `simd`", WHY_SIMD));
        };
        let item = item.clone();
        let plain = |node: NodeKind, value_type: Type| RawCall {
            node: Node { span, kind: node },
            value_type,
            failures: Vec::new(),
            unnamed: None,
        };
        let receiver = Box::new(receiver);
        let takes_nothing = |arguments: &[Argument]| -> Outcome<()> {
            match arguments.first() {
                Some(extra) => Err(Report::refused(
                    extra.value.span,
                    format!("`.%{}` takes nothing", name.text),
                    WHY_SIMD,
                )),
                None => Ok(()),
            }
        };
        match (base, narrowing) {
            ("sum" | "product" | "min_val" | "max_val" | "all_true" | "any_true", None) => {
                takes_nothing(arguments)?;
                Self::simd_reduction(receiver, receiver_type, &item, name, span)
            }
            ("pick", None) if item == Type::Bool => self.check_lane_pick(*receiver, lanes, arguments, span),
            ("reverse", None) => {
                takes_nothing(arguments)?;
                let from: Vec<usize> = (0..lanes).rev().collect();
                Ok(plain(NodeKind::Shuffle { value: receiver, from }, receiver_type.clone()))
            }
            ("rotate", Some(direction @ ("left" | "right"))) => {
                let by = Self::fixed_count(arguments, "by", span)?;
                let by = by % lanes;
                let from: Vec<usize> = (0..lanes)
                    .map(|lane| {
                        if direction == "left" {
                            (lane + by) % lanes
                        } else {
                            (lane + lanes - by) % lanes
                        }
                    })
                    .collect();
                Ok(plain(NodeKind::Shuffle { value: receiver, from }, receiver_type.clone()))
            }
            ("rotate", None) => Err(Report::refused(
                name.span,
                "`rotate` says its direction",
                "`.%rotate.left(by: 1)` or `.%rotate.right(by: 1)`, so no one guesses which way a number turns (D307)",
            )),
            ("join" | "interleave", None) => {
                self.check_join_lanes(receiver, receiver_type, name, arguments, span)
            }
            ("shuffle", None) => {
                let (from, _) = Self::shuffle_list(arguments, lanes, span)?;
                let count = from.len();
                Ok(plain(
                    NodeKind::Shuffle { value: receiver, from },
                    Type::Simd {
                        item: Box::new(item),
                        lanes: count,
                    },
                ))
            }
            ("lookup", None) => self.check_lookup(*receiver, receiver_type, arguments, span),
            (other, Some(narrowing)) => Err(Report::refused(
                name.span,
                format!("a `simd` has no `.%{other}.{narrowing}`"),
                WHY_SIMD,
            )),
            (other, None) => Err(Report::refused(
                name.span,
                format!("a `simd` has no method `{other}`"),
                "a `simd` combines its lanes with `sum`, `product`, `min_val`, `max_val`, `all_true` and `any_true`, chooses with `pick`, and reorders with `shuffle`, `lookup`, `reverse`, `rotate`, `join` and `interleave` (D297, D302, D304, D306, D307)",
            )),
        }
    }

    /// `.%sum()`, `.%product()`, `.%min_val()` and `.%max_val()` on number lanes, and
    /// `.%all_true()` and `.%any_true()` on a `simd<bool>`. (D297)
    fn simd_reduction(
        receiver: Box<Node>,
        receiver_type: &Type,
        item: &Type,
        name: &crate::ast::Name,
        span: Span,
    ) -> Outcome<RawCall> {
        let boolean = matches!(name.text.as_str(), "all_true" | "any_true");
        let number = lane_number(item);
        if boolean != (*item == Type::Bool) || (!boolean && number.is_none()) {
            return Err(Report::refused(
                name.span,
                format!("`.%{}` doesn't work on `{receiver_type}`", name.text),
                "number lanes combine with `sum`, `product`, `min_val` and `max_val`, and a `simd<bool>` with `all_true` and `any_true` (D297)",
            ));
        }
        let how = match name.text.as_str() {
            "sum" => Reduction::Sum,
            "product" => Reduction::Product,
            "min_val" => Reduction::Min,
            "max_val" => Reduction::Max,
            "all_true" => Reduction::AllTrue,
            _ => Reduction::AnyTrue,
        };
        Ok(RawCall {
            node: Node {
                span,
                kind: NodeKind::Reduce {
                    how,
                    number,
                    value: receiver,
                },
            },
            value_type: if boolean { Type::Bool } else { item.clone() },
            failures: Vec::new(),
            unnamed: None,
        })
    }

    /// `$a.%join(then: $b)` puts two vectors end to end, and `$a.%interleave(with: $b)`
    /// alternates their lanes, with the longer one's last lanes following on. (D307)
    fn check_join_lanes(
        &mut self,
        receiver: Box<Node>,
        receiver_type: &Type,
        name: &crate::ast::Name,
        arguments: &[Argument],
        span: Span,
    ) -> Outcome<RawCall> {
        let why = "`$a.%join(then: $b)` puts two vectors end to end, and `$a.%interleave(with: $b)` alternates their lanes (D307)";
        let label = if name.text == "join" { "then" } else { "with" };
        let [other] = arguments else {
            return Err(Report::refused(
                span,
                format!("`.%{}` takes `{label}:`", name.text),
                why,
            ));
        };
        if other.label.as_ref().is_none_or(|found| found.text != label) {
            return Err(Report::refused(
                other.value.span,
                format!("`.%{}` takes `{label}:`", name.text),
                why,
            ));
        }
        let Some((item, lanes)) = simd_parts(receiver_type) else {
            return Err(Report::refused(span, "this isn't a `simd`", WHY_SIMD));
        };
        let (other_node, other_type) = self.check_expression(&other.value, Some(receiver_type))?;
        let Some((other_item, other_lanes)) = simd_parts(&other_type) else {
            return Err(Report::refused(
                other.value.span,
                format!("this is `{other_type}`, not a `simd`"),
                WHY_SIMD,
            ));
        };
        if other_item != item {
            return Err(Report::refused(
                other.value.span,
                format!("this mixes `{receiver_type}` and `{other_type}`"),
                WHY_SIMD,
            ));
        }
        let total = lanes + other_lanes;
        let joined = Node {
            span,
            kind: NodeKind::JoinLanes {
                left: receiver,
                right: Box::new(other_node),
            },
        };
        let mut node = joined;
        if name.text == "interleave" {
            let mut from = Vec::with_capacity(total);
            for lane in 0..lanes.max(other_lanes) {
                if lane < lanes {
                    from.push(lane);
                }
                if lane < other_lanes {
                    from.push(lanes + lane);
                }
            }
            node = Node {
                span,
                kind: NodeKind::Shuffle {
                    value: Box::new(node),
                    from,
                },
            };
        }
        Ok(RawCall {
            node,
            value_type: Type::Simd {
                item: Box::new(item.clone()),
                lanes: total,
            },
            failures: Vec::new(),
            unnamed: None,
        })
    }

    /// `by: 1`, a count fixed when compiling, like a shuffle list. (D307)
    fn fixed_count(arguments: &[Argument], label: &str, span: Span) -> Outcome<usize> {
        let [argument] = arguments else {
            return Err(Report::refused(
                span,
                format!("this takes `{label}:`"),
                "`by:` is fixed when compiling (D307)",
            ));
        };
        if argument
            .label
            .as_ref()
            .is_none_or(|found| found.text != label)
        {
            return Err(Report::refused(
                argument.value.span,
                format!("this takes `{label}:`"),
                "`by:` is fixed when compiling (D307)",
            ));
        }
        super::expression::literal_integer(&argument.value)
            .and_then(|count| usize::try_from(count).ok())
            .ok_or_else(|| {
                Report::refused(
                    argument.value.span,
                    "`by:` is a whole number fixed when compiling",
                    "`rotate`'s `by:` is fixed when compiling, like a shuffle list, since CPUs bake it into the instruction (D307)",
                )
            })
    }

    /// `pos: [4, 3, 2, 1]` or `offset.lanes: [3, 2, 1, 0]`: for each lane of the result, which
    /// lane it comes from, fixed when compiling. (D304, D305)
    /// trace: D304, D305, D306
    fn shuffle_list(
        arguments: &[Argument],
        lanes: usize,
        span: Span,
    ) -> Outcome<(Vec<usize>, bool)> {
        let why = "a shuffle lists, for each lane of the result, which lane it comes from, with `pos:` counting from 1 or `offset.lanes:` from 0, fixed when compiling (D304, D305)";
        let [argument] = arguments else {
            return Err(Report::refused(
                span,
                "a shuffle takes `pos:` or `offset.lanes:`",
                why,
            ));
        };
        let from_zero = match argument.label.as_ref().map(|label| label.text.as_str()) {
            Some("pos") => false,
            Some("offset.lanes") => true,
            Some("offset") => {
                let label = argument.label.as_ref().map_or(span, |label| label.span);
                return Err(Report::refused(
                    label,
                    "a bare `offset:` could read as bytes, so it says `offset.lanes:`",
                    "a shuffle counting from 0 says `offset.lanes:`, matching the type's `lanes:` (D305)",
                )
                .fix("offset.lanes"));
            }
            _ => {
                return Err(Report::refused(
                    argument.value.span,
                    "a shuffle takes `pos:` or `offset.lanes:`",
                    why,
                ))
            }
        };
        let ExpressionKind::Items(items) = &argument.value.kind else {
            return Err(Report::refused(
                argument.value.span,
                "a shuffle's list is fixed when compiling, so lanes worked out while running use `.%lookup`",
                "CPUs bake a shuffle's pattern into the instruction, and a table lookup is a slower, separate thing (D304, D306)",
            )
            .fix(".%lookup"));
        };
        let mut from = Vec::with_capacity(items.len());
        for item in items {
            let Some(number) = super::expression::literal_integer(item) else {
                return Err(Report::refused(
                    item.span,
                    "a shuffle's list is fixed when compiling, so lanes worked out while running use `.%lookup`",
                    "CPUs bake a shuffle's pattern into the instruction, and a table lookup is a slower, separate thing (D304, D306)",
                )
                .fix(".%lookup"));
            };
            let lane = if from_zero { number } else { number - 1 };
            let Some(lane) = usize::try_from(lane).ok().filter(|lane| *lane < lanes) else {
                return Err(Report::refused(
                    item.span,
                    format!(
                        "this `simd` has {lanes} lanes, so there's no lane {}{number}",
                        if from_zero { "at offset " } else { "" }
                    ),
                    why,
                ));
            };
            from.push(lane);
        }
        if from.is_empty() {
            return Err(Report::refused(
                argument.value.span,
                "a shuffle gives at least one lane",
                why,
            ));
        }
        Ok((from, from_zero))
    }

    /// `.%lookup(pos: $which)`: lanes from positions worked out while running, where one past
    /// the end is a bounds failure that says which lane. (D306)
    /// trace: D306
    fn check_lookup(
        &mut self,
        table: Node,
        table_type: &Type,
        arguments: &[Argument],
        span: Span,
    ) -> Outcome<RawCall> {
        let why = "`.%lookup(pos: $which)` takes its lanes from positions in another `simd`, worked out while running (D306)";
        let [argument] = arguments else {
            return Err(Report::refused(
                span,
                "`.%lookup` takes `pos:` or `offset.lanes:`",
                why,
            ));
        };
        let counts_from_zero = match argument.label.as_ref().map(|label| label.text.as_str()) {
            Some("pos") => false,
            Some("offset.lanes") => true,
            Some("offset") => {
                return Err(Report::refused(
                    argument.label.as_ref().map_or(span, |label| label.span),
                    "a bare `offset:` could read as bytes, so it says `offset.lanes:`",
                    "counting from 0 says `offset.lanes:` (D305)",
                )
                .fix("offset.lanes"));
            }
            _ => {
                return Err(Report::refused(
                    argument.value.span,
                    "`.%lookup` takes `pos:` or `offset.lanes:`",
                    why,
                ))
            }
        };
        let (which_node, which_type) = self.check_expression(&argument.value, None)?;
        let Some((Type::Integer { .. }, count)) = simd_parts(&which_type) else {
            return Err(Report::refused(
                argument.value.span,
                format!(
                    "`.%lookup`'s positions are a `simd` of integers, and this is `{which_type}`"
                ),
                why,
            ));
        };
        let Some((item, _)) = simd_parts(table_type) else {
            return Err(Report::refused(span, "this isn't a `simd`", WHY_SIMD));
        };
        Ok(RawCall {
            node: Node {
                span,
                kind: NodeKind::Lookup {
                    table: Box::new(table),
                    which: Box::new(which_node),
                    counts_from_zero,
                },
            },
            value_type: Type::Simd {
                item: Box::new(item.clone()),
                lanes: count,
            },
            failures: Vec::new(),
            unnamed: None,
        })
    }

    /// `.%pick(if_true:, if_false:)` on a `simd<bool>`: each lane from one of two vectors,
    /// with both worked out in full. (D302)
    /// trace: D302
    fn check_lane_pick(
        &mut self,
        condition: Node,
        lanes: usize,
        arguments: &[Argument],
        span: Span,
    ) -> Outcome<RawCall> {
        let why = "`.%pick(if_true:, if_false:)` chooses each lane from one of two vectors, both worked out in full (D302)";
        let [if_true, if_false] = arguments else {
            return Err(Report::refused(
                span,
                "`.%pick` takes `if_true:` and `if_false:`",
                why,
            ));
        };
        let labelled = |argument: &Argument, label: &str| {
            argument
                .label
                .as_ref()
                .is_some_and(|found| found.text == label)
        };
        if !labelled(if_true, "if_true") || !labelled(if_false, "if_false") {
            return Err(Report::refused(
                span,
                "`.%pick` takes `if_true:` and `if_false:`, in that order",
                why,
            ));
        }
        let (true_node, true_type) = self.check_expression(&if_true.value, None)?;
        let (false_node, false_type) = self.check_expression(&if_false.value, Some(&true_type))?;
        let Some((_, true_lanes)) = simd_parts(&true_type) else {
            return Err(Report::not_designed(
                if_true.value.span,
                format!("`.%pick` on a `simd<bool>` between two `{true_type}`, which aren't `simd` values"),
                "",
            ));
        };
        if true_type != false_type || true_lanes != lanes {
            return Err(Report::refused(
                if_false.value.span,
                format!("`.%pick` on {lanes} lanes chooses between two of the same `simd`, and this has `{true_type}` and `{false_type}`"),
                why,
            ));
        }
        Ok(RawCall {
            node: Node {
                span,
                kind: NodeKind::LanePick {
                    condition: Box::new(condition),
                    if_true: Box::new(true_node),
                    if_false: Box::new(false_node),
                },
            },
            value_type: true_type,
            failures: Vec::new(),
            unnamed: None,
        })
    }

    /// `as` between an array and a `simd` of the same count, both ways, which can't fail.
    /// (D298)
    /// trace: D298
    pub(super) fn simd_as(found: &Type, target: &Type, span: Span) -> Option<Outcome<()>> {
        let (array_item, array_len, simd_item, lanes) = match (found, target) {
            (Type::Array { item, len, .. }, Type::Simd { item: lane, lanes })
            | (Type::Simd { item: lane, lanes }, Type::Array { item, len, .. }) => {
                (item, *len, lane, *lanes)
            }
            _ => return None,
        };
        if array_item != simd_item {
            return Some(Err(Report::refused(
                span,
                format!("`{found}` and `{target}` hold different types"),
                "`as` converts between an array and a `simd` of the same item type and count (D298)",
            )));
        }
        if usize::try_from(array_len).ok() != Some(lanes) {
            return Some(Err(Report::refused(
                span,
                format!("`{found}` and `{target}` have different counts"),
                "`as` converts between an array and a `simd` of the same count, so it can't fail (D298)",
            )));
        }
        Some(Ok(()))
    }
}
