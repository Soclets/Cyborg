//! C's union, `union.@csup::untagged`, whose fields share the same bytes. It's built by naming
//! the one field written, and any field can be read, which nothing checks against the one
//! written, all inside shields. (D33, D38, D398, D399)
//!
//! The oracle holds the bytes the written field fills, least significant first, as on every
//! target, and reads another field from them. Reading more bytes than were written isn't said.

use super::Checker;
use crate::ast::{FieldValue, Name};
use crate::ir::{Node, NodeKind};
use crate::report::{Outcome, Report, Span};
use crate::types::Type;

const WHY_SHIELD: &str = "C's union is declared, built and read only in shields, where the reason vouches for it (D38, D398)";

impl Checker<'_> {
    /// `number[f = 1.5]`: exactly one field, the one written.
    /// trace: D398
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
        let [written] = fields else {
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
        Ok((
            Node {
                span,
                kind: NodeKind::Encode(Box::new(node), field_type.clone()),
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
