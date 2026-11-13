//! `simd` (D291 to D307, D403): a portable vector of lanes. The checker works lane by lane, so every lane follows its
//! type's rules exactly and a failure says which lane (D293), and LLVM puts the lanes back into vector instructions.

use super::expressions::{Typed, When, failure};
use super::types::Type;
use super::{Body, Check, Checker, Reported};
use crate::checked::{self, ExpressionKind as Kind, LocalId, MachineType};
use crate::diagnostic::{Diagnostic, Kind as DiagnosticKind};
use crate::source::Span;
use crate::syntax::{self, BinaryOperator, ExpressionKind};

const SHUFFLE_WHY: &str = "a shuffle lists, for each lane of the result, which lane it comes from, with `pos:` counting from 1 or `offset.lanes:` from 0, fixed when compiling (D304, D305)";
const NOT_DECIDED_SIMD: &str = "not decided yet (standard 12.2)";

/// A lane of a value held in a local.
fn lane(local: LocalId, index: u64, item: &Type) -> checked::Expression {
    checked::Expression {
        kind: Kind::Load(checked::Place {
            local,
            steps: vec![checked::Step::Item {
                position: Box::new(checked::Expression {
                    kind: Kind::Integer(i128::from(index)),
                    machine_type: MachineType::I64,
                }),
                from_one: false,
                check: None,
            }],
        }),
        machine_type: item.machine(),
    }
}

/// `value` held in `local` while `body` is worked out.
fn bind(local: LocalId, value: checked::Expression, body: checked::Expression) -> checked::Expression {
    let machine_type = body.machine_type.clone();
    checked::Expression { kind: Kind::Bind { local, value: Box::new(value), body: Box::new(body) }, machine_type }
}

/// A `simd` value built from its lanes.
fn build(lanes: Vec<checked::Expression>, value_type: &Type) -> checked::Expression {
    checked::Expression { kind: Kind::Build(lanes), machine_type: value_type.machine() }
}

impl Checker<'_> {
    /// `simd<i32>[lanes: 4]`, or `lanes: max`, the target's baseline register width over the lane's size (D291, D292,
    /// D403).
    pub fn simd_type(
        &mut self,
        written: &syntax::Type,
        fill: &[syntax::TypeArgument],
        labels: &[syntax::Label],
    ) -> Check<Type> {
        let [syntax::TypeArgument { label: None, value: syntax::TypeArgumentValue::Type(item), .. }] = fill else {
            return Err(self.not_yet(written.span, "`simd` filled in this way".to_string()));
        };
        let item = self.resolve_type(item)?;
        let [label] = labels else {
            return Err(self.not_yet(written.span, "`simd` without one `lanes:`".to_string()));
        };
        if label.name.text() != "lanes" {
            return Err(self.not_yet(label.span, format!("`{}` on `simd`", label.name.text())));
        }
        let lanes = match &label.value.kind {
            ExpressionKind::Path(path) if path.is_single("max") => {
                if !crate::target::named() {
                    // trace: D292, D389
                    return Err(self.error(Diagnostic::refused(
                        label.value.span,
                        "how many lanes `max` is depends on the target, which this run doesn't name",
                        "`lanes: max` is the most one instruction works on for the target being built, so a program using it names its target with `--target=` (D292, D389)",
                    )));
                }
                // trace: D403
                let register = if self.target.pointer_bytes() == 4 { 32 } else { 128 };
                let lane_bits = match &item {
                    Type::Bool => 8,
                    other => self.footprint(other).map_or(32, |footprint| footprint.size * 8),
                };
                (register / lane_bits.max(1)).max(1)
            }
            _ => {
                let count = self.constant_integer(&label.value)?;
                u64::try_from(count).ok().filter(|count| *count > 0).ok_or_else(|| {
                    self.error(Diagnostic::refused(
                        label.value.span,
                        "a `simd` has at least one lane",
                        "`lanes:` takes any count from 1, or `max` (D292)",
                    ))
                })?
            }
        };
        Ok(Type::Simd { item: Box::new(item), lanes })
    }

    /// An operator on two `simd` values works on every lane (D291, D293).
    pub fn simd_arithmetic(
        &mut self,
        body: &mut Body,
        expression: &syntax::Expression,
        operator: &BinaryOperator,
        operator_span: Span,
        (left, right): (Typed, Typed),
    ) -> Check<Typed> {
        let (Type::Simd { item, lanes }, Type::Simd { item: other, lanes: other_lanes }) =
            (&left.value_type, &right.value_type)
        else {
            return Err(self.error(Diagnostic::not_designed(
                expression.span,
                format!("an operator between `{}` and `{}`, a `simd` and one value", left.value_type, right.value_type),
                NOT_DECIDED_SIMD,
            )));
        };
        if item != other || lanes != other_lanes {
            return Err(self.error(Diagnostic::refused(
                expression.span,
                format!("this mixes `{}` and `{}`", left.value_type, right.value_type),
                "`as` builds a new value of another type, and `:` doesn't convert (D7)",
            )));
        }
        let (item, lanes) = ((**item).clone(), *lanes);
        let value_type = left.value_type.clone();
        let left_local = body.new_local(value_type.machine());
        let right_local = body.new_local(value_type.machine());
        let mut built = Vec::new();
        let mut lane_type = item.clone();
        for index in 0..lanes {
            let a = Typed::new(item.clone(), lane(left_local, index, &item).kind);
            let b = Typed::new(item.clone(), lane(right_local, index, &item).kind);
            self.lane = Some(index + 1);
            let result = self.arithmetic_on(expression, operator, operator_span, (a, b));
            self.lane = None;
            let result = result?;
            lane_type = result.value_type.clone();
            built.push(result.value);
        }
        let value_type = Type::Simd { item: Box::new(lane_type), lanes };
        let inner = bind(right_local, right.value, build(built, &value_type));
        Ok(Typed::new(value_type, bind(left_local, left.value, inner).kind))
    }

    /// A comparison on two `simd` values gives one `bool` per lane (D301).
    pub fn simd_comparison(
        &mut self,
        body: &mut Body,
        expression: &syntax::Expression,
        operator: &BinaryOperator,
        (left, right): (Typed, Typed),
    ) -> Check<Typed> {
        let (Type::Simd { item, lanes }, Type::Simd { item: other, lanes: other_lanes }) =
            (&left.value_type, &right.value_type)
        else {
            return Err(self.error(Diagnostic::not_designed(
                expression.span,
                format!("comparing `{}` and `{}`, a `simd` and one value", left.value_type, right.value_type),
                NOT_DECIDED_SIMD,
            )));
        };
        if item != other || lanes != other_lanes {
            return Err(self.error(Diagnostic::refused(
                expression.span,
                format!("this compares `{}` and `{}`", left.value_type, right.value_type),
                "`as` builds a new value of another type, and `:` doesn't convert (D7)",
            )));
        }
        let (item, lanes) = ((**item).clone(), *lanes);
        let left_local = body.new_local(left.value_type.machine());
        let right_local = body.new_local(right.value_type.machine());
        let a = Typed::new(item.clone(), Kind::Zero);
        let comparison = self.comparison(expression.span, operator, &a, &a)?;
        let built = (0..lanes)
            .map(|index| checked::Expression {
                kind: Kind::Compare {
                    operator: comparison,
                    left: Box::new(lane(left_local, index, &item)),
                    right: Box::new(lane(right_local, index, &item)),
                },
                machine_type: MachineType::Bool,
            })
            .collect();
        let value_type = Type::Simd { item: Box::new(Type::Bool), lanes };
        let inner = bind(right_local, right.value, build(built, &value_type));
        Ok(Typed::new(value_type, bind(left_local, left.value, inner).kind))
    }

    /// `$v.%sum()` and the other methods on a `simd` (D297, D302, D304 to D307).
    #[allow(clippy::too_many_lines)]
    pub fn simd_method(
        &mut self,
        body: &mut Body,
        expression: &syntax::Expression,
        receiver: &syntax::Expression,
        written: &str,
        arguments: &[syntax::Argument],
    ) -> Check<Typed> {
        let typed = self.expression(body, receiver, None)?;
        let Type::Simd { item, lanes } = typed.value_type.clone() else {
            if typed.value_type.is_unknown() {
                return Err(Reported);
            }
            return Err(self.not_yet(expression.span, format!("a method on `{}`", typed.value_type)));
        };
        let item = *item;
        let local = body.new_local(typed.value_type.machine());
        let lanes_of = |count: u64| (0..count).map(|index| lane(local, index, &item)).collect::<Vec<_>>();
        let named = |arguments: &[syntax::Argument], label: &str| {
            arguments.iter().find(|argument| argument.label.as_ref().is_some_and(|each| each.text == label)).cloned()
        };
        let result: Typed = match written {
            // trace: D296, D297
            "%sum" | "%product" => {
                let operator = if written == "%sum" { BinaryOperator::Add } else { BinaryOperator::Multiply };
                let mut level: Vec<Typed> =
                    lanes_of(lanes).into_iter().map(|value| Typed::new(item.clone(), value.kind)).collect();
                while level.len() > 1 {
                    let mut next = Vec::new();
                    let mut pending = level.into_iter();
                    while let Some(a) = pending.next() {
                        match pending.next() {
                            Some(b) => next.push(self.arithmetic_on(expression, &operator, expression.span, (a, b))?),
                            None => next.push(a),
                        }
                    }
                    level = next;
                }
                level.pop().ok_or(Reported)?
            }
            "%min_val" | "%max_val" => {
                let a = Typed::new(item.clone(), Kind::Zero);
                let operator = if written == "%min_val" { BinaryOperator::Below } else { BinaryOperator::Above };
                let comparison = self.comparison(expression.span, &operator, &a, &a)?;
                let mut values = lanes_of(lanes).into_iter();
                let mut best = values.next().ok_or(Reported)?;
                for value in values {
                    let held = body.new_local(item.machine());
                    let kept = body.new_local(item.machine());
                    let read = |local| checked::Expression { kind: Kind::Local(local), machine_type: item.machine() };
                    let pick = checked::Expression {
                        kind: Kind::Select {
                            condition: Box::new(checked::Expression {
                                kind: Kind::Compare {
                                    operator: comparison,
                                    left: Box::new(read(held)),
                                    right: Box::new(read(kept)),
                                },
                                machine_type: MachineType::Bool,
                            }),
                            then: Box::new(read(held)),
                            otherwise: Box::new(read(kept)),
                        },
                        machine_type: item.machine(),
                    };
                    best = bind(kept, best, bind(held, value, pick));
                }
                Typed::new(item.clone(), best.kind)
            }
            "%all_true" | "%any_true" => {
                if item != Type::Bool {
                    return Err(self.error(Diagnostic::refused(
                        expression.span,
                        format!("`{written}` is for `simd<bool>`, and this is `{}`", typed.value_type),
                        "`.%all_true()` and `.%any_true()` combine the answers of a `simd<bool>` (D297)",
                    )));
                }
                let all = written == "%all_true";
                let mut values = lanes_of(lanes).into_iter();
                let mut combined = values.next().ok_or(Reported)?;
                for value in values {
                    let kind = if all {
                        Kind::And(Box::new(combined), Box::new(value))
                    } else {
                        Kind::Or(Box::new(combined), Box::new(value))
                    };
                    combined = checked::Expression { kind, machine_type: MachineType::Bool };
                }
                Typed::new(Type::Bool, combined.kind)
            }
            // trace: D302
            "%pick" => {
                if item != Type::Bool {
                    return Err(self.not_yet(expression.span, "`.%pick` on a `simd` that isn't of `bool`".to_string()));
                }
                let (Some(when_true), Some(when_false)) = (named(arguments, "if_true"), named(arguments, "if_false"))
                else {
                    return Err(
                        self.not_yet(expression.span, "`.%pick` without `if_true:` and `if_false:`".to_string())
                    );
                };
                let a = self.expression(body, &when_true.value, None)?;
                let b = self.expression(body, &when_false.value, None)?;
                let Type::Simd { item: picked, lanes: picked_lanes } = a.value_type.clone() else {
                    return Err(self.not_yet(when_true.span, "`.%pick` between values that aren't `simd`".to_string()));
                };
                if b.value_type != a.value_type || picked_lanes != lanes {
                    return Err(self.error(Diagnostic::refused(
                        expression.span,
                        format!("this picks between `{}` and `{}`", a.value_type, b.value_type),
                        "`.%pick` chooses each lane from one of two `simd` values with the same lanes (D302)",
                    )));
                }
                let (a_local, b_local) =
                    (body.new_local(a.value_type.machine()), body.new_local(b.value_type.machine()));
                let built = (0..lanes)
                    .map(|index| checked::Expression {
                        kind: Kind::Select {
                            condition: Box::new(lane(local, index, &Type::Bool)),
                            then: Box::new(lane(a_local, index, &picked)),
                            otherwise: Box::new(lane(b_local, index, &picked)),
                        },
                        machine_type: picked.machine(),
                    })
                    .collect();
                let value_type = a.value_type.clone();
                let inner = bind(b_local, b.value, build(built, &value_type));
                Typed::new(value_type, bind(a_local, a.value, inner).kind)
            }
            // trace: D304, D305, D306, D307
            "%shuffle" => {
                let positions = self.shuffle_positions(arguments, lanes)?;
                let value_type = Type::Simd { item: Box::new(item.clone()), lanes: positions.len() as u64 };
                let built = positions.iter().map(|position| lane(local, *position, &item)).collect();
                Typed::new(value_type.clone(), build(built, &value_type).kind)
            }
            "%reverse" => {
                let built = (0..lanes).rev().map(|index| lane(local, index, &item)).collect();
                Typed::new(typed.value_type.clone(), build(built, &typed.value_type).kind)
            }
            "%rotate.left" | "%rotate.right" => {
                let Some(by) = named(arguments, "by") else {
                    return Err(self.not_yet(expression.span, "a rotate without `by:`".to_string()));
                };
                let by = u64::try_from(self.constant_integer(&by.value)?).unwrap_or(0) % lanes;
                let start = if written == "%rotate.left" { by } else { lanes - by };
                let built = (0..lanes).map(|index| lane(local, (index + start) % lanes, &item)).collect();
                Typed::new(typed.value_type.clone(), build(built, &typed.value_type).kind)
            }
            "%join" | "%interleave" => {
                let label = if written == "%join" { "then" } else { "with" };
                let Some(other) = named(arguments, label) else {
                    return Err(self.not_yet(expression.span, format!("`{written}` without `{label}:`")));
                };
                let other = self.expression(body, &other.value, None)?;
                let Type::Simd { item: other_item, lanes: other_lanes } = other.value_type.clone() else {
                    return Err(self.not_yet(expression.span, format!("`{written}` with something that isn't `simd`")));
                };
                if *other_item != item {
                    return Err(self.error(Diagnostic::refused(
                        expression.span,
                        format!("this joins `{}` and `{}`", typed.value_type, other.value_type),
                        "`join` and `interleave` put two vectors of the same lane type together (D307)",
                    )));
                }
                let other_local = body.new_local(other.value_type.machine());
                let mut built = Vec::new();
                if written == "%join" {
                    built.extend(lanes_of(lanes));
                    built.extend((0..other_lanes).map(|index| lane(other_local, index, &item)));
                } else {
                    for index in 0..lanes.max(other_lanes) {
                        if index < lanes {
                            built.push(lane(local, index, &item));
                        }
                        if index < other_lanes {
                            built.push(lane(other_local, index, &item));
                        }
                    }
                }
                let value_type = Type::Simd { item: Box::new(item.clone()), lanes: lanes + other_lanes };
                Typed::new(value_type.clone(), bind(other_local, other.value, build(built, &value_type)).kind)
            }
            "%lookup" => self.lookup(body, expression, local, &item, lanes, arguments)?,
            other => return Err(self.not_yet(expression.span, format!("the method `{other}` on a `simd`"))),
        };
        Ok(Typed::new(result.value_type, bind(local, typed.value, result.value).kind))
    }

    /// The lanes a `.%shuffle` names, fixed when compiling (D304, D305, D306).
    fn shuffle_positions(&mut self, arguments: &[syntax::Argument], lanes: u64) -> Check<Vec<u64>> {
        let [argument] = arguments else {
            return Err(self.not_yet(
                arguments.first().map_or(Span::default(), |first| first.span),
                "a shuffle with these arguments".to_string(),
            ));
        };
        let label = argument.label.as_ref().map(|label| label.text.clone()).unwrap_or_default();
        let from_one = match label.as_str() {
            "pos" => true,
            "offset.lanes" => false,
            "offset" => {
                let span = argument.label.as_ref().map_or(argument.span, |label| label.span);
                return Err(self.error(Diagnostic::refused_fix(
                    span,
                    "offset.lanes",
                    "a shuffle counting from 0 says `offset.lanes:`, matching the type's `lanes:` (D305)",
                )));
            }
            _ => return Err(self.not_yet(argument.span, "a shuffle without `pos:` or `offset.lanes:`".to_string())),
        };
        let ExpressionKind::List(items) = &argument.value.kind else {
            return Err(self.error(Diagnostic::refused_fix(
                argument.value.span,
                ".%lookup",
                "CPUs bake a shuffle's pattern into the instruction, and a table lookup is a slower, separate thing (D304, D306)",
            )));
        };
        let mut positions = Vec::new();
        for item in items {
            let ExpressionKind::Integer(text) = &item.kind else {
                return Err(self.error(Diagnostic::refused_fix(
                    item.span,
                    ".%lookup",
                    "CPUs bake a shuffle's pattern into the instruction, and a table lookup is a slower, separate thing (D304, D306)",
                )));
            };
            let written: u64 = text.replace('_', "").parse().unwrap_or(u64::MAX);
            let index = if from_one { written.checked_sub(1) } else { Some(written) };
            match index {
                Some(index) if index < lanes => positions.push(index),
                _ => {
                    return Err(self.error(Diagnostic::refused(
                        item.span,
                        format!("this `simd` has {lanes} lanes, so there's no lane {written}"),
                        SHUFFLE_WHY,
                    )));
                }
            }
        }
        Ok(positions)
    }

    /// `$table.%lookup(pos: $which)`: lanes from positions worked out while running, each checked (D306).
    fn lookup(
        &mut self,
        body: &mut Body,
        expression: &syntax::Expression,
        table: LocalId,
        item: &Type,
        lanes: u64,
        arguments: &[syntax::Argument],
    ) -> Check<Typed> {
        let [argument] = arguments else {
            return Err(self.not_yet(expression.span, "`.%lookup` with these arguments".to_string()));
        };
        let label = argument.label.as_ref().map(|label| label.text.clone()).unwrap_or_default();
        let from_one = match label.as_str() {
            "pos" => true,
            "offset.lanes" => false,
            _ => return Err(self.not_yet(argument.span, "`.%lookup` without `pos:` or `offset.lanes:`".to_string())),
        };
        let which = self.expression(body, &argument.value, None)?;
        let Type::Simd { item: position_type, lanes: count } = which.value_type.clone() else {
            return Err(self.not_yet(argument.span, "`.%lookup` with positions that aren't a `simd`".to_string()));
        };
        let Type::Integer { kind, .. } = *position_type else {
            return Err(self.not_yet(argument.span, "`.%lookup` with positions that aren't whole numbers".to_string()));
        };
        let which_local = body.new_local(which.value_type.machine());
        let position_machine = position_type.machine();
        let (low, high) = if from_one { (1, i128::from(lanes)) } else { (0, i128::from(lanes) - 1) };
        let mut built = Vec::new();
        for index in 0..count {
            let shown = if from_one { "pos" } else { "offset.lanes" };
            let check = failure(
                self,
                Diagnostic::new(
                    DiagnosticKind::CheckFailed,
                    expression.span,
                    format!("`{shown}: \u{1}` is outside the table"),
                    "a lane of `.%lookup` pointing past the end is a bounds failure that says which lane (D306)",
                ),
                &[When::Text(format!("in lane {}, and the table has {lanes} lanes", index + 1))],
            );
            let position = checked::Expression {
                kind: Kind::Fit {
                    value: Box::new(lane(which_local, index, &position_type)),
                    min: low,
                    max: high,
                    failure: Box::new(check),
                },
                machine_type: position_machine.clone(),
            };
            let _ = kind;
            built.push(checked::Expression {
                kind: Kind::Load(checked::Place {
                    local: table,
                    steps: vec![checked::Step::Item { position: Box::new(position), from_one, check: None }],
                }),
                machine_type: item.machine(),
            });
        }
        let value_type = Type::Simd { item: Box::new(item.clone()), lanes: count };
        Ok(Typed::new(value_type.clone(), bind(which_local, which.value, build(built, &value_type)).kind))
    }
}
