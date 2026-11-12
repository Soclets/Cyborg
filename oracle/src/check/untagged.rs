//! C's union, `union.@csup::untagged`, whose fields share the same bytes. It's built by naming
//! the one field written, and any field can be read, which nothing checks against the one
//! written, all inside shields. (D33, D38, D398, D399)
//!
//! The build says what the rest of its bytes are, `rest = all 0`, so the oracle holds the
//! written field's bytes, least significant first, as on every target, then zeros up to the
//! widest field, and reads any field from them, checked like any value of its type. (D401,
//! D402)

use super::Checker;
use crate::ast::{ExpressionKind, FieldValue, Name};
use crate::ir::{Node, NodeKind};
use crate::report::{Outcome, Report, ReportExtra, Span};
use crate::types::Type;

const WHY_SHIELD: &str = "C's union is declared, built and read only in shields, where the reason vouches for it (D38, D398)";

impl Checker<'_> {
    /// `number[f = 1.5, rest = all 0]`: exactly one field, the one written, and what the rest of
    /// the bytes are.
    /// trace: D398, D401, D402
    pub(super) fn build_untagged(
        &mut self,
        union: &str,
        fields: &[FieldValue],
        span: Span,
    ) -> Outcome<(Node, Type)> {
        if self.context.in_shield == 0 {
            return Err(Report::refused(
                span,
                format!("building a `union.@csup::untagged {union}` outside a shield"),
                WHY_SHIELD,
            ));
        }
        let (rest, fields): (Vec<&FieldValue>, Vec<&FieldValue>) =
            fields.iter().partition(|field| field.name.text == "rest");
        let Some(rest) = rest.first() else {
            return Err(Report::refused(
                span,
                format!("this builds `{union}` without saying what the rest of its bytes are"),
                "the bytes a union's written field doesn't cover are a choice, so the build says it, and `rest = all 0` sets them to 0 (D401, D402)",
            )
            .fix_elsewhere("rest = all 0"));
        };
        match &rest.value.kind {
            ExpressionKind::All(inner) if super::expression::literal_integer(inner) == Some(0) => {}
            ExpressionKind::Empty => {
                return Err(Report::refused(
                    rest.value.span,
                    "`empty` means a thing holds nothing, and a union's bytes always exist",
                    "a union fills the rest of its bytes with `rest = all 0`, as `all 0` fills an array (D402, D103)",
                )
                .fix("all 0"));
            }
            _ => {
                return Err(Report::not_designed(
                    rest.value.span,
                    "the rest of a union's bytes as anything but `all 0`",
                    "",
                ));
            }
        }
        let [written] = fields.as_slice() else {
            return Err(Report::refused(
                span,
                format!("this builds `{union}` with {} fields, and C's union is built by naming exactly the one written", fields.len()),
                "a C union's fields share the same bytes, so building one names the field it writes (D398)",
            ));
        };
        let declared = self.untagged.get(union).cloned().unwrap_or_default();
        let Some((_, field_type)) = declared
            .iter()
            .find(|(field, _)| *field == written.name.text)
        else {
            return Err(Report::refused(
                written.name.span,
                format!("`{union}` has no field `{}`", written.name.text),
                "a C union is built by naming one of its fields (D398)",
            ));
        };
        let (node, found) = self.check_expression(&written.value, Some(field_type))?;
        let node = self.coerce(node, &found, field_type, &written.value)?;
        let size = declared
            .iter()
            .filter_map(|(_, field)| field_bytes(field))
            .max()
            .unwrap_or(0);
        Ok((
            Node {
                span,
                kind: NodeKind::Encode(Box::new(node), field_type.clone(), size),
            },
            Type::Untagged(union.to_string()),
        ))
    }

    /// `$n.i`: any field, read from the bytes the union holds.
    /// trace: D398
    pub(super) fn read_untagged(
        &self,
        node: Node,
        union: &str,
        name: &Name,
        span: Span,
    ) -> Outcome<(Node, Type)> {
        if self.context.in_shield == 0 {
            return Err(Report::refused(
                span,
                format!("reading a `union.@csup::untagged {union}` outside a shield"),
                WHY_SHIELD,
            ));
        }
        let declared = self.untagged.get(union).cloned().unwrap_or_default();
        let Some((_, field_type)) = declared.iter().find(|(field, _)| *field == name.text) else {
            return Err(Report::refused(
                name.span,
                format!("`{union}` has no field `{}`", name.text),
                "a C union's fields are the ones it declares (D398)",
            ));
        };
        Ok((
            Node {
                span,
                kind: NodeKind::Decode(Box::new(node), field_type.clone()),
            },
            field_type.clone(),
        ))
    }
}

/// How many bytes a field takes.
pub(crate) fn field_bytes(field: &Type) -> Option<usize> {
    match field {
        Type::Integer { kind, .. } => usize::try_from(kind.bits() / 8).ok(),
        Type::Float { kind, .. } => Some(match kind {
            crate::types::FloatKind::F32 => 4,
            crate::types::FloatKind::F64 => 8,
        }),
        Type::Bool => Some(1),
        _ => None,
    }
}
