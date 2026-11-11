//! Expressions.

use crate::checked;
use crate::diagnostic::Diagnostic;
use crate::syntax::{self, ExpressionKind};

use super::types::Type;
use super::{Body, Check, Checker};

impl Checker<'_> {
    /// Checks a value, with the type it's going into when that's known, since a literal takes its type from where it
    /// lands (D134).
    pub fn expression(
        &mut self,
        body: &mut Body,
        expression: &syntax::Expression,
        expected: Option<&Type>,
    ) -> Check<(Type, checked::Expression)> {
        match &expression.kind {
            ExpressionKind::Integer(text) => {
                let value_type = match expected {
                    Some(value_type @ Type::Integer { .. }) => value_type.plain(),
                    _ => Type::I32,
                };
                let value: i128 = text
                    .replace('_', "")
                    .parse()
                    .map_err(|_| self.not_yet(expression.span, "a number this long".to_string()))?;
                Ok((
                    value_type.clone(),
                    checked::Expression {
                        kind: checked::ExpressionKind::Integer(value),
                        machine_type: value_type.machine(),
                    },
                ))
            }
            ExpressionKind::Variable(name) => {
                let Some(variable) = body.lookup(&name.text).cloned() else {
                    return Err(self.not_yet(name.span, format!("`{}` isn't declared", name.text)));
                };
                let machine = variable.value_type.machine();
                Ok((
                    variable.value_type,
                    checked::Expression { kind: checked::ExpressionKind::Local(variable.local), machine_type: machine },
                ))
            }
            ExpressionKind::Group(inner) => self.expression(body, inner, expected),
            _ => Err(self.not_yet(expression.span, "this kind of value".to_string())),
        }
    }

    /// A value going into a place of type `into`, checked to fit.
    pub fn value_into(
        &mut self,
        body: &mut Body,
        expression: &syntax::Expression,
        into: &Type,
    ) -> Check<checked::Expression> {
        let (value_type, value) = self.expression(body, expression, Some(into))?;
        if value_type.is_unknown() || into.is_unknown() {
            return Ok(value);
        }
        if value_type.plain() != into.plain() {
            return Err(self.error(Diagnostic::refused(
                expression.span,
                format!("this is `{value_type}`, and it goes into `{into}`"),
                "a value goes where its type does",
            )));
        }
        Ok(value)
    }
}
