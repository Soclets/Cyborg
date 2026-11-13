//! Secrets (D339 to D342, D361, D362): an integer or `bool` narrowed with `.secret` mustn't affect how long the program
//! takes, so the checker refuses whatever could branch on one, or touch memory or take time by its value. A secret
//! mixed with a plain value gives a secret, and only `.%reveal()`, in a shield, makes one public.

use super::expressions::Typed;
use super::types::Type;
use super::{Body, Check, Checker, Reported};
use crate::checked::ExpressionKind as Kind;
use crate::diagnostic::Diagnostic;
use crate::source::Span;
use crate::syntax::{self, BinaryOperator};

/// Why a secret can't go where a plain value goes (D340, D342).
pub const PUBLIC_WHY: &str = "a secret goes where a plain value is expected only through `.%reveal()`, inside a shield, so it can't be lost by accident (D340, D342)";

/// Takes any secrecy off a checked value, saying whether it had it.
fn public(mut typed: Typed) -> (Typed, bool) {
    match typed.value_type {
        Type::Secret(inner) => {
            typed.value_type = *inner;
            (typed, true)
        }
        _ => (typed, false),
    }
}

/// Puts secrecy on a checked value.
pub fn secret(mut typed: Typed, is_secret: bool) -> Typed {
    if is_secret && !matches!(typed.value_type, Type::Secret(_) | Type::Unknown) {
        typed.value_type = Type::Secret(Box::new(typed.value_type));
    }
    typed
}

impl Checker<'_> {
    /// Maths or bits on a secret: what takes the same time for every value, with secrecy spreading to the result, and
    /// refused otherwise (D340, D341).
    pub fn secret_arithmetic(
        &mut self,
        expression: &syntax::Expression,
        operator: &BinaryOperator,
        operator_span: Span,
        (left, right): (Typed, Typed),
    ) -> Check<Typed> {
        let (left, left_secret) = public(left);
        let (right, right_secret) = public(right);
        match operator {
            BinaryOperator::Add | BinaryOperator::Subtract | BinaryOperator::Multiply => {
                // trace: D341
                let symbol = self.slice(operator_span).to_string();
                let word = match operator {
                    BinaryOperator::Add => "plus",
                    BinaryOperator::Subtract => "minus",
                    _ => "times",
                };
                return Err(self.error(Diagnostic::refused(
                    expression.span,
                    format!("a plain `{symbol}` on a secret checks for overflow, which is a branch"),
                    format!(
                        "on a secret, maths is written `{word}.wrap` or `{word}.saturate`, which never branch, since crashing on overflow would tell an attacker one bit about the secret (D341)"
                    ),
                )));
            }
            BinaryOperator::Divide(_) | BinaryOperator::Remainder(_) => {
                // trace: D340
                return Err(self.error(Diagnostic::refused(
                    expression.span,
                    "`div` and `mod` take longer for some values on most CPUs",
                    "a secret mustn't affect how long the program takes, so `div` and `mod` with a secret on either side are refused (D340)",
                )));
            }
            BinaryOperator::ShiftLeft
            | BinaryOperator::ShiftRight
            | BinaryOperator::RotateLeft
            | BinaryOperator::RotateRight
                if right_secret =>
            {
                return Err(self.error(Diagnostic::refused(
                    expression.span,
                    "this shifts by a secret amount",
                    "some CPUs take longer for bigger shifts, so a shift's amount stays public (D340)",
                )));
            }
            _ => {}
        }
        let result = self.arithmetic_on(expression, operator, operator_span, (left, right))?;
        Ok(secret(result, left_secret || right_secret))
    }

    /// A comparison with a secret on either side gives a `bool.secret` (D340).
    pub fn secret_comparison(
        &mut self,
        expression: &syntax::Expression,
        operator: &BinaryOperator,
        (left, right): (Typed, Typed),
    ) -> Check<Typed> {
        let (left, _) = public(left);
        let (right, _) = public(right);
        let comparison = self.comparison(expression.span, operator, &left, &right)?;
        let compared = Typed::new(
            Type::Bool,
            Kind::Compare { operator: comparison, left: Box::new(left.value), right: Box::new(right.value) },
        );
        Ok(secret(compared, true))
    }

    /// A `bool` for `not`, `and` and `or`, which may be a secret, since those don't branch when both sides run (D362).
    pub fn logic_value(&mut self, body: &mut Body, expression: &syntax::Expression) -> Check<(Typed, bool)> {
        let typed = self.expression(body, expression, Some(&Type::Bool))?;
        match &typed.value_type {
            Type::Secret(inner) if **inner == Type::Bool => Ok(public(typed)),
            _ => {
                let typed = self.condition_value_of(expression, typed)?;
                Ok((typed, false))
            }
        }
    }

    /// `$cipher.%reveal()`: a secret made public, only in a shield (D342).
    pub fn reveal(
        &mut self,
        body: &mut Body,
        expression: &syntax::Expression,
        receiver: &syntax::Expression,
    ) -> Check<Typed> {
        if self.shields == 0 {
            // trace: D342
            return Err(self.error(Diagnostic::refused(
                expression.span,
                "a secret becomes public only inside a shield",
                "only the programmer knows a secret is safe to make public, a claim the compiler can't check, so `.%reveal()` lives in a shield whose reason says why (D342)",
            )));
        }
        let typed = self.expression(body, receiver, None)?;
        if typed.value_type.is_unknown() {
            return Err(Reported);
        }
        Ok(public(typed).0)
    }
}
