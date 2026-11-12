//! Secrets: `.secret` on an integer or a `bool`, which mustn't affect how long the program
//! takes. (D339, D340, D341, D342, D361, D362)
//!
//! A secret is `Type::Secret` around its plain type, so anything that doesn't know about it
//! refuses it, and only what takes the same time for every value lets it through: the bit
//! operators, the word forms of `+`, `-` and `x`, shifts by a public amount, comparisons, and
//! `.%pick`. A secret mixed with a plain value gives a secret.

use super::expression::RawCall;
use super::Checker;
use crate::ast::{Argument, BinaryOperator, Expression};
use crate::ir::{Node, NodeKind};
use crate::report::{Outcome, Report, ReportExtra, Span};
use crate::types::Type;

/// The plain type inside a secret, and whether it was one.
pub(super) fn unwrap_secret(value_type: &Type) -> (Type, bool) {
    match value_type {
        Type::Secret(inner) => ((**inner).clone(), true),
        other => (other.clone(), false),
    }
}

/// Wraps a plain integer or `bool` as a secret.
pub(super) fn secret(value_type: Type) -> Type {
    match value_type {
        Type::Secret(_) => value_type,
        other => Type::Secret(Box::new(other)),
    }
}

/// The refusal for anything whose timing or path depends on a secret. (D340)
pub(super) fn refuse_on_a_secret(span: Span, what: impl Into<String>, why: &str) -> Box<Report> {
    Report::refused(span, what, why)
}

/// Why a branch on a secret is refused. (D340)
pub(super) const WHY_BRANCH: &str = "a secret mustn't affect how long the program takes, and the path an `if`, `while` or `match` takes leaks it (D339, D340)";

impl Checker<'_> {
    /// Which binary operators a secret allows, before its types are worked out: plain `+`,
    /// `-` and `x` check for overflow, which is a branch, and `div` and `mod` take longer for
    /// some values. (D340, D341)
    /// trace: D340, D341
    pub(super) fn refuse_secret_operator(
        operator: BinaryOperator,
        left_secret: bool,
        right_secret: bool,
        span: Span,
    ) -> Outcome<()> {
        if !left_secret && !right_secret {
            return Ok(());
        }
        match operator {
            BinaryOperator::Add | BinaryOperator::Subtract | BinaryOperator::Multiply => {
                let word = match operator {
                    BinaryOperator::Add => "plus",
                    BinaryOperator::Subtract => "minus",
                    _ => "times",
                };
                Err(refuse_on_a_secret(
                    span,
                    format!("a plain `{}` on a secret checks for overflow, which is a branch", match operator {
                        BinaryOperator::Add => "+",
                        BinaryOperator::Subtract => "-",
                        _ => "x",
                    }),
                    &format!("on a secret, maths is written `{word}.wrap` or `{word}.saturate`, which never branch, since crashing on overflow would tell an attacker one bit about the secret (D341)"),
                ))
            }
            BinaryOperator::Divide(_) | BinaryOperator::Remainder(_) => Err(refuse_on_a_secret(
                span,
                "`div` and `mod` take longer for some values on most CPUs",
                "a secret mustn't affect how long the program takes, so `div` and `mod` with a secret on either side are refused (D340)",
            )),
            BinaryOperator::ShiftLeft | BinaryOperator::ShiftRight if right_secret => {
                Err(refuse_on_a_secret(
                    span,
                    "this shifts by a secret amount",
                    "some CPUs take longer for bigger shifts, so a shift's amount stays public (D340)",
                ))
            }
            _ => Ok(()),
        }
    }

    /// `$same.%pick(if_true: $x, if_false: $y)` works out both sides and takes no branch, the
    /// constant-time choice. `$cipher.%reveal()` makes a secret public, inside a shield, whose
    /// reason says why. (D340, D342)
    /// trace: D340, D342
    pub(super) fn check_secret_method(
        &mut self,
        receiver_node: Node,
        receiver_type: &Type,
        name: &crate::ast::Name,
        arguments: &[Argument],
        span: Span,
    ) -> Outcome<RawCall> {
        let (inner, _) = unwrap_secret(receiver_type);
        match name.text.as_str() {
            "reveal" => {
                if let Some(extra) = arguments.first() {
                    return Err(Report::refused(
                        extra.value.span,
                        "`.%reveal()` takes nothing",
                        "`.%reveal()` makes a secret public (D342)",
                    ));
                }
                if self.context.in_shield == 0 {
                    return Err(Report::refused(
                        span,
                        "a secret becomes public only inside a shield",
                        "only the programmer knows a secret is safe to make public, a claim the compiler can't check, so `.%reveal()` lives in a shield whose reason says why (D342)",
                    ));
                }
                Ok(RawCall {
                    node: receiver_node,
                    value_type: inner,
                    failures: Vec::new(),
                    unnamed: None,
                })
            }
            "pick" if inner == Type::Bool => self.check_pick(receiver_node, arguments, span),
            other => Err(Report::refused(
                name.span,
                format!("`{receiver_type}` has no method `{other}`"),
                "a secret has `.%pick(if_true:, if_false:)` on a `bool.secret`, and `.%reveal()` inside a shield (D340, D342)",
            )),
        }
    }

    /// Both sides of a `.%pick`, which have the same type, and the result is secret. (D340)
    fn check_pick(
        &mut self,
        condition: Node,
        arguments: &[Argument],
        span: Span,
    ) -> Outcome<RawCall> {
        let why =
            "`.%pick(if_true:, if_false:)` works out both sides and takes no branch (D340, D302)";
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
        let (false_node, false_type) =
            self.check_expression(&if_false.value, Some(&unwrap_secret(&true_type).0))?;
        let (true_plain, _) = unwrap_secret(&true_type);
        let (false_plain, _) = unwrap_secret(&false_type);
        let false_node = self.coerce(false_node, &false_plain, &true_plain, &if_false.value)?;
        if !matches!(true_plain, Type::Integer { .. } | Type::Bool) {
            return Err(Report::not_designed(
                if_true.value.span,
                format!("`.%pick` between two `{true_plain}`"),
                "",
            ));
        }
        Ok(RawCall {
            node: Node {
                span,
                kind: NodeKind::Pick {
                    condition: Box::new(condition),
                    if_true: Box::new(true_node),
                    if_false: Box::new(false_node),
                },
            },
            value_type: secret(true_plain),
            failures: Vec::new(),
            unnamed: None,
        })
    }

    /// An `if`, `while` or `match` on a secret, or a secret as a position, is refused. (D340)
    pub(super) fn refuse_secret_branch(found: &Type, expression: &Expression) -> Outcome<()> {
        if matches!(found, Type::Secret(_)) {
            return Err(refuse_on_a_secret(
                expression.span,
                format!("this branches on `{found}`"),
                WHY_BRANCH,
            ));
        }
        Ok(())
    }

    /// Plain `and` and `or` stop early, which is a branch, so a secret uses `and.both` and
    /// `or.both`. (D362)
    /// trace: D362
    pub(super) fn refuse_secret_early_stop(
        operator: BinaryOperator,
        secret_side: Option<&Expression>,
    ) -> Outcome<()> {
        let Some(side) = secret_side else {
            return Ok(());
        };
        let word = match operator {
            BinaryOperator::And => "and",
            BinaryOperator::Or => "or",
            _ => return Ok(()),
        };
        Err(refuse_on_a_secret(
            side.span,
            format!("plain `{word}` stops early, which branches on this secret"),
            "on a `bool.secret`, `and.both` and `or.both` always run both sides, so nothing branches (D362)",
        )
        .fix(format!("{word}.both")))
    }

    /// `if $key == 0:` and the rest: what an `if` or `while` tests can't be secret. (D340)
    pub(super) fn refuse_secret_condition(
        &self,
        condition_type: &Type,
        condition: &Expression,
    ) -> Outcome<()> {
        if !matches!(condition_type, Type::Secret(_)) {
            return Ok(());
        }
        let shown = self.text_of(condition.span).to_string();
        Err(refuse_on_a_secret(
            condition.span,
            format!("`{shown}` is a `{condition_type}`, and branching on it leaks it"),
            WHY_BRANCH,
        ))
    }

    /// A print, or anything that wants a plain value, can't take a secret. (D340)
    pub(super) fn refuse_secret_print(found: &Type, value: &Expression) -> Outcome<()> {
        if matches!(found, Type::Secret(_)) {
            return Err(refuse_on_a_secret(
                value.span,
                format!("this is `{found}`, and printing it would make it public"),
                "a secret goes where a plain value is expected only through `.%reveal()`, inside a shield (D340, D342)",
            ));
        }
        Ok(())
    }
}
