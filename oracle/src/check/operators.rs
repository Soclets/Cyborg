//! A type's own operators: `+`, `-`, `x`, the `div` narrowings, `==` and `below`, with `!=`, `above`,
//! `at_least` and `at_most` following from the last two. Each becomes a call to the function
//! the type wrote, with its values lent, since the operator shows no word. (D236, D237)

use super::{Checker, OperatorInfo};
use crate::ast::{
    Argument, BinaryOperator, CompareOperator, Expression, ExpressionKind, HandWord, Narrowing,
    Path,
};
use crate::ir::{Node, NodeKind};
use crate::report::{Outcome, Report, Span};
use crate::types::Type;

/// The name a type's own operator is declared under, for an arithmetic operator.
pub(super) fn arithmetic_word(operator: BinaryOperator) -> Option<&'static str> {
    match operator {
        BinaryOperator::Add => Some("+"),
        BinaryOperator::Subtract => Some("-"),
        BinaryOperator::Multiply => Some("x"),
        BinaryOperator::Divide(Narrowing::Trunc) => Some("div.trunc"),
        BinaryOperator::Divide(Narrowing::Floor) => Some("div.floor"),
        BinaryOperator::Divide(Narrowing::Exact) => Some("div.exact"),
        _ => None,
    }
}

/// Whether an expression is made only of number literals, like `(16 div.trunc 1)`, which can
/// never be a struct or union.
fn only_number_literals(expression: &Expression) -> bool {
    match &expression.kind {
        ExpressionKind::Group(inner) | ExpressionKind::Negate(inner) => only_number_literals(inner),
        ExpressionKind::Binary { left, right, .. } => {
            only_number_literals(left) && only_number_literals(right)
        }
        _ => super::expression::is_number_literal(expression),
    }
}

/// An integer or float with no unit.
fn plain_number(found: &Type) -> bool {
    matches!(
        found,
        Type::Integer { unit: None, .. } | Type::Float { unit: None, .. }
    )
}

/// A struct or union, which can have operators of its own. (D236)
pub(super) fn is_own(found: &Type) -> bool {
    matches!(found, Type::Struct(_) | Type::Union(_))
}

impl Checker<'_> {
    /// An expression's type read from its form alone, without checking it, so that choosing a
    /// type's operator doesn't check its values twice. Nothing when the form doesn't say.
    pub(super) fn type_from_form(&self, expression: &Expression) -> Option<Type> {
        match &expression.kind {
            ExpressionKind::Variable(name) => {
                let slot = self.find_variable(name)?;
                let variable = self.context.variables.get(slot)?;
                Some(match &variable.held_type {
                    Type::Borrow { inner, .. } => (**inner).clone(),
                    other => other.clone(),
                })
            }
            ExpressionKind::Group(inner) => self.type_from_form(inner),
            ExpressionKind::Field { base, name } => {
                let Type::Struct(owner) = self.type_from_form(base)? else {
                    return None;
                };
                let info = self.struct_info(&owner)?;
                info.fields
                    .iter()
                    .find(|(field, _)| *field == name.text)
                    .map(|(_, found)| found.clone())
            }
            ExpressionKind::Build { path, .. } => self
                .structs
                .contains_key(&path.text)
                .then(|| Type::Struct(path.text.clone())),
            // A generic's result is its type parameter filled from the arguments, and when
            // they don't say, the form doesn't either. (D241)
            // trace: D241
            ExpressionKind::Call { path, arguments } => {
                let signature = self.signatures.get(&self.canonical(&path.text))?;
                if signature.type_parameters.is_empty() {
                    return Some(signature.return_type.clone());
                }
                self.fill_type_parameters(signature, arguments, None, expression.span)
                    .ok()
                    .map(|(filled, _)| filled.return_type)
            }
            ExpressionKind::Binary {
                operator,
                left,
                right,
            } => {
                // A number literal is never a struct or union, so next to a plain number it's
                // built-in maths, since a type's own operator has its own type on a side. (D236)
                // trace: D236
                let literal = |side: &Expression| only_number_literals(side);
                let left_literal = literal(left);
                let right_literal = literal(right);
                let left = self.type_from_form(left);
                let right = self.type_from_form(right);
                let built_in = (left_literal && right.as_ref().is_some_and(plain_number))
                    || (right_literal && left.as_ref().is_some_and(plain_number))
                    || (left_literal && right_literal);
                if let (false, Some(word)) = (built_in, arithmetic_word(*operator)) {
                    if let Some(found) = self.find_operator(word, left.as_ref(), right.as_ref()) {
                        return Some(found.result.clone());
                    }
                }
                // A built-in operator on a plain number keeps its type, which a shift takes
                // from its left side. A unit's `x` and `div` make another unit, so only
                // numbers with none are read. (D241)
                // trace: D241
                let known = match operator {
                    BinaryOperator::ShiftLeft | BinaryOperator::ShiftRight => left,
                    BinaryOperator::And
                    | BinaryOperator::Or
                    | BinaryOperator::AndBoth
                    | BinaryOperator::OrBoth => return None,
                    _ => left.or(right),
                }?;
                plain_number(&known).then(|| super::generics::unbounded(&known))
            }
            ExpressionKind::Negate(inner) => self
                .type_from_form(inner)
                .filter(plain_number)
                .map(|found| super::generics::unbounded(&found)),
            _ => None,
        }
    }

    /// The one operator named `word` whose values' types fit the sides that are known.
    fn find_operator(
        &self,
        word: &str,
        left: Option<&Type>,
        right: Option<&Type>,
    ) -> Option<&OperatorInfo> {
        if left.is_none() && right.is_none() {
            return None;
        }
        let mut fitting = self.operators.iter().filter(|found| {
            found.operator == word
                && left.is_none_or(|left| *left == found.left)
                && right.is_none_or(|right| *right == found.right)
        });
        let first = fitting.next()?;
        fitting.next().is_none().then_some(first)
    }

    /// Whether the type has its own operator `word` for these two sides.
    pub(super) fn has_operator(&self, word: &str, left: &Expression, right: &Expression) -> bool {
        let left = self.type_from_form(left);
        let right = self.type_from_form(right);
        self.find_operator(word, left.as_ref(), right.as_ref())
            .is_some()
    }

    /// Whether either side is a struct or union, so the operator is the type's own, or a type
    /// parameter, so it's what the call's table says. (D236, D235)
    pub(super) fn sides_are_own(&self, left: &Expression, right: &Expression) -> bool {
        let own = |found: Option<Type>| {
            found
                .as_ref()
                .is_some_and(|found| is_own(found) || matches!(found, Type::Parameter(_)))
        };
        own(self.type_from_form(left)) || own(self.type_from_form(right))
    }

    /// `$a + $b` on a type of your own: a call to its `op +`. (D236)
    /// trace: D236
    pub(super) fn own_arithmetic(
        &mut self,
        word: &str,
        left: &Expression,
        right: &Expression,
        span: Span,
    ) -> Outcome<(Node, Type)> {
        self.call_operator(word, word, (left, right), span)
    }

    /// One comparison on a type of your own. `==` and `below` are the type's own, and the
    /// others follow from them: `!=` is not `==`, `above` is `below` the other way round,
    /// `at_least` is not `below`, and `at_most` is not `below` the other way round. (D237, D278)
    /// trace: D237
    pub(super) fn own_comparison(
        &mut self,
        operator: CompareOperator,
        left: &Expression,
        right: &Expression,
        span: Span,
    ) -> Outcome<(Node, Type)> {
        let written = operator.word();
        let (word, sides, negated) = match operator {
            CompareOperator::Equal => ("==", (left, right), false),
            CompareOperator::NotEqual => ("==", (left, right), true),
            CompareOperator::Below => ("below", (left, right), false),
            CompareOperator::Above => ("below", (right, left), false),
            CompareOperator::AtLeast => ("below", (left, right), true),
            CompareOperator::AtMost => ("below", (right, left), true),
        };
        let (node, found) = self.call_operator(word, written, sides, span)?;
        if negated {
            return Ok((
                Node {
                    span,
                    kind: NodeKind::Not(Box::new(node)),
                },
                found,
            ));
        }
        Ok((node, found))
    }

    /// A chain like `$a < $b < $c` on a type of your own, as each pair joined by `and`. A
    /// value in the middle is read twice, so it has to be a variable or a field. (D117, D237)
    /// trace: D117, D237
    pub(super) fn own_comparison_chain(
        &mut self,
        first: &Expression,
        rest: &[(CompareOperator, Expression)],
        span: Span,
    ) -> Outcome<(Node, Type)> {
        let mut left = first;
        let mut joined: Option<Node> = None;
        for (index, (operator, right)) in rest.iter().enumerate() {
            if index + 1 < rest.len() && !is_a_place(right) {
                return Err(Report::not_designed(
                    right.span,
                    "a chain of comparisons on your own type with a worked-out value in the middle",
                    "",
                ));
            }
            let (node, _) = self.own_comparison(*operator, left, right, span)?;
            joined = Some(match joined {
                Some(before) => Node {
                    span,
                    kind: NodeKind::And(Box::new(before), Box::new(node)),
                },
                None => node,
            });
            left = right;
        }
        let Some(node) = joined else {
            return Err(Report::refused(span, "this comparison is empty", "(D49)"));
        };
        Ok((node, Type::Bool))
    }

    /// Calls the type's operator `word` on the two sides, each lent when it's a struct or
    /// union, as the operator's values are. `written` is what the program wrote, for the error
    /// when there's no such operator.
    fn call_operator(
        &mut self,
        word: &str,
        written: &str,
        (left, right): (&Expression, &Expression),
        span: Span,
    ) -> Outcome<(Node, Type)> {
        // trace: D235
        if self.is_generic(left) || self.is_generic(right) {
            return self.generic_operator(word, written, (left, right), span);
        }
        let left_type = self.type_from_form(left);
        let right_type = self.type_from_form(right);
        let Some(found) = self
            .find_operator(word, left_type.as_ref(), right_type.as_ref())
            .cloned()
        else {
            let shown = |found: Option<&Type>| {
                found.map_or("this".to_string(), |found| format!("`{found}`"))
            };
            let follows = if word == written {
                String::new()
            } else {
                format!(", which `{written}` follows from")
            };
            return Err(Report::refused(
                span,
                format!(
                    "there's no `op {word}` for {} and {}{follows}",
                    shown(left_type.as_ref()),
                    shown(right_type.as_ref())
                ),
                "a struct or union gets an operator by defining it with `op`, and it applies only where its values' types match (D236, D237)",
            ));
        };
        // A fresh value has no holder, so it takes no word. (D2)
        let lend = |value: &Expression, declared: &Type| -> Argument {
            let value = if is_own(declared) && is_a_place(value) {
                Expression {
                    span: value.span,
                    kind: ExpressionKind::Hand {
                        word: HandWord::Lend,
                        value: Box::new(value.clone()),
                    },
                }
            } else {
                value.clone()
            };
            Argument { label: None, value }
        };
        let call = Expression {
            span,
            kind: ExpressionKind::Call {
                path: Path {
                    text: found.function.clone(),
                    span,
                },
                arguments: vec![lend(left, &found.left), lend(right, &found.right)],
            },
        };
        self.check_expression(&call, None)
    }
}

/// A variable or a field of one, which reading twice gives the same value.
fn is_a_place(expression: &Expression) -> bool {
    match &expression.kind {
        ExpressionKind::Variable(_) => true,
        ExpressionKind::Field { base, .. } | ExpressionKind::Group(base) => is_a_place(base),
        _ => false,
    }
}
