//! Raw and volatile pointers, which only live in a shield. (D314, D315, D316, D317, D226)
//!
//! `.of` points at a variable, which stays lent for as long as the pointer exists, and reading
//! and writing through it work. `.at` points at an address, which the oracle has no memory at,
//! so reading or writing through one isn't built here.

use super::expression::RawCall;
use super::Checker;
use crate::ast::{Argument, Expression, ExpressionKind, HandWord, Path, TypeExpression};
use crate::ir::{Node, NodeKind};
use crate::report::{Outcome, Report, ReportExtra, Span};
use crate::types::Type;

const WHY_SHIELD: &str = "raw and volatile pointers live only in a shield, where the programmer takes over what the compiler can't check (D38, D226, D314)";

/// Whether a type is a pointer that can write.
pub(super) fn is_writing_pointer(value_type: &Type) -> bool {
    matches!(value_type, Type::Pointer { mutable: true, .. })
}

impl Checker<'_> {
    /// `ptr.raw<T>`, `ptr.raw.mut<T>`, `ptr.volatile<T>` or `ptr.volatile.mut<T>`. (D314, D317)
    /// trace: D314, D317
    pub(super) fn pointer_kind(family: &str, span: Span) -> Outcome<(bool, bool)> {
        match family {
            "ptr.raw" => Ok((false, false)),
            "ptr.raw.mut" => Ok((false, true)),
            "ptr.volatile" => Ok((true, false)),
            "ptr.volatile.mut" => Ok((true, true)),
            other => Err(Report::refused(
                span,
                format!("`{other}` isn't a kind of pointer"),
                "a pointer is `ptr.raw<T>` or `ptr.volatile<T>`, with `.mut` when it can write (D314, D317)",
            )),
        }
    }

    pub(super) fn refuse_outside_a_shield(&self, span: Span, what: &str) -> Outcome<()> {
        if self.context.in_shield == 0 {
            return Err(Report::refused(
                span,
                format!("{what} lives only in a shield"),
                WHY_SHIELD,
            ));
        }
        Ok(())
    }

    /// A pointer type, which only a shield can hold.
    pub(super) fn resolve_pointer_type(
        &self,
        family: &str,
        target: &TypeExpression,
        span: Span,
    ) -> Outcome<Type> {
        let (volatile, mutable) = Self::pointer_kind(family, span)?;
        self.refuse_outside_a_shield(span, "a pointer")?;
        Ok(Type::Pointer {
            volatile,
            mutable,
            target: Box::new(self.resolve_type(target)?),
        })
    }

    /// `ptr.raw.mut<u32>.at(address: 0x4004_0000)` makes a pointer from a number, and
    /// `ptr.raw.mut<i32>.of(Lend.mut $count)` from a variable, lent with the word that says the
    /// access: `Lend` to read and `Lend.mut` to write. (D315)
    /// trace: D315
    pub(super) fn check_pointer(
        &mut self,
        family: &Path,
        target: &TypeExpression,
        how: &crate::ast::Name,
        arguments: &[Argument],
        span: Span,
    ) -> Outcome<(Node, Type)> {
        self.refuse_outside_a_shield(span, "a pointer")?;
        let pointer_type = self.resolve_pointer_type(&family.text, target, family.span)?;
        let Type::Pointer {
            mutable,
            target: target_type,
            ..
        } = &pointer_type
        else {
            return Err(Report::refused(span, "this isn't a pointer", WHY_SHIELD));
        };
        let [argument] = arguments else {
            return Err(Report::refused(
                span,
                format!("`.{}` takes one argument", how.text),
                WHY_SHIELD,
            ));
        };
        match how.text.as_str() {
            "at" => {
                if argument
                    .label
                    .as_ref()
                    .is_none_or(|label| label.text != "address")
                {
                    return Err(Report::refused(
                        argument.value.span,
                        "`.at` takes `address:`",
                        "`.at(address: 0x4004_0000)` makes a pointer from a number, for hardware registers and memory maps (D315)",
                    ));
                }
                let (node, found) = self.check_expression(
                    &argument.value,
                    Some(&Type::plain_integer(crate::types::IntegerKind::U64)),
                )?;
                if !matches!(found, Type::Integer { unit: None, .. }) {
                    return Err(Report::refused(
                        argument.value.span,
                        format!("an address is a plain integer, and this is `{found}`"),
                        "`.at(address: 0x4004_0000)` makes a pointer from a number (D315)",
                    ));
                }
                Ok((
                    Node {
                        span,
                        kind: NodeKind::PointerAt(Box::new(node)),
                    },
                    pointer_type,
                ))
            }
            "of" => self.check_pointer_of(
                family,
                argument,
                *mutable,
                target_type,
                pointer_type.clone(),
                span,
            ),
            other => Err(Report::refused(
                how.span,
                format!("a pointer is made with `.at` or `.of`, not `.{other}`"),
                "`.at(address:)` makes one from a number and `.of` from a variable (D315)",
            )),
        }
    }

    /// `.of(Lend.mut $count)`: a pointer to a variable, lent with the word that says the access.
    /// (D315)
    fn check_pointer_of(
        &mut self,
        family: &Path,
        argument: &Argument,
        mutable: bool,
        target_type: &Type,
        pointer_type: Type,
        span: Span,
    ) -> Outcome<(Node, Type)> {
        let wanted = if mutable {
            HandWord::LendMut
        } else {
            HandWord::Lend
        };
        let ExpressionKind::Hand { word, value } = &argument.value.kind else {
            return Err(Report::refused(
                argument.value.span,
                "`.of` takes a variable with the word for its access",
                "`.of` takes `Lend` for a pointer that reads and `Lend.mut` for one that writes (D315)",
            )
            .fix(format!("{} {}", wanted.word(), self.text_of(argument.value.span))));
        };
        if *word != wanted {
            return Err(Report::refused(
                argument.value.span,
                format!("`{family}` is lent with `{}`", wanted.word(), family = family.text),
                "`.of` takes `Lend` for a pointer that reads and `Lend.mut` for one that writes (D315)",
            )
            .fix(format!("{} {}", wanted.word(), self.text_of(value.span))));
        }
        let held = self.check_lent_place(value, wanted)?;
        if held.value_type != *target_type {
            return Err(Report::refused(
                value.span,
                format!(
                    "this is `{}`, and the pointer is to `{target_type}`",
                    held.value_type
                ),
                "a pointer to `T` points at a `T` (D314)",
            ));
        }
        let Some(place) = held.place else {
            return Err(Report::refused(
                value.span,
                "`.of` points at a variable",
                "(D315)",
            ));
        };
        Ok((
            Node {
                span,
                kind: NodeKind::PointerOf(place),
            },
            pointer_type,
        ))
    }

    /// `$p.%read()`, `$p.%%write(val: 1)` and `$p.%moved(offset.items: 1)`. (D315, D316)
    /// trace: D315, D316
    pub(super) fn check_pointer_method(
        &mut self,
        pointer: Node,
        pointer_type: &Type,
        name: &crate::ast::Name,
        changes: bool,
        arguments: &[Argument],
        span: Span,
    ) -> Outcome<RawCall> {
        self.refuse_outside_a_shield(span, "a pointer")?;
        let Type::Pointer {
            mutable, target, ..
        } = pointer_type
        else {
            return Err(Report::refused(span, "this isn't a pointer", WHY_SHIELD));
        };
        let plain = |kind: NodeKind, value_type: Type| RawCall {
            node: Node { span, kind },
            value_type,
            failures: Vec::new(),
            unnamed: None,
        };
        match name.text.as_str() {
            "read" => {
                if changes {
                    return Err(Self::wrong_receiver_mark(name, false));
                }
                if let Some(extra) = arguments.first() {
                    return Err(Report::refused(
                        extra.value.span,
                        "`.%read()` takes nothing",
                        "(D315)",
                    ));
                }
                Ok(plain(
                    NodeKind::PointerRead(Box::new(pointer)),
                    (**target).clone(),
                ))
            }
            "write" => {
                if !mutable {
                    return Err(Report::refused(
                        span,
                        format!("`{pointer_type}` can only read"),
                        "writing through a pointer needs `ptr.raw.mut` or `ptr.volatile.mut`, since even in a shield that mistake is cheap to catch (D314, D317)",
                    ));
                }
                if !changes {
                    return Err(Self::wrong_receiver_mark(name, true));
                }
                let [value] = arguments else {
                    return Err(Report::refused(span, "`.%%write` takes `val:`", "(D315)"));
                };
                if value.label.as_ref().is_none_or(|label| label.text != "val") {
                    return Err(Report::refused(
                        value.value.span,
                        "`.%%write` takes `val:`",
                        "(D315)",
                    ));
                }
                let node = self.check_into(&value.value, target)?;
                Ok(plain(
                    NodeKind::PointerWrite {
                        pointer: Box::new(pointer),
                        value: Box::new(node),
                    },
                    Type::Void,
                ))
            }
            "moved" => self.check_moved(pointer, pointer_type, target, arguments, span),
            other => Err(Report::refused(
                name.span,
                format!("a pointer has no method `{other}`"),
                "a pointer has `.%read()`, `.%%write(val:)` and `.%moved(…)` (D315, D316)",
            )),
        }
    }

    /// `.%moved(offset.items: 1)` or `.%moved(offset.bytes: 2)`: a new pointer, its unit always
    /// written, and nothing checked. (D316)
    /// trace: D316
    fn check_moved(
        &mut self,
        pointer: Node,
        pointer_type: &Type,
        target: &Type,
        arguments: &[Argument],
        span: Span,
    ) -> Outcome<RawCall> {
        let [argument] = arguments else {
            return Err(Report::refused(
                span,
                "`.%moved` takes `offset.items:` or `offset.bytes:`",
                "(D316)",
            ));
        };
        let label = argument.label.as_ref().map(|label| label.text.as_str());
        let item_bytes = match (label, target) {
            (Some("offset.bytes"), _) => 1,
            (Some("offset.items"), Type::Integer { kind, .. }) => i128::from(kind.bits() / 8),
            (Some("offset.items"), _) => {
                return Err(Report::not_built(
                    argument.value.span,
                    format!("moving a pointer to `{target}` by items"),
                ));
            }
            (Some("offset"), _) => {
                return Err(Report::refused(
                    argument.label.as_ref().map_or(span, |label| label.span),
                    "a bare `offset:` hides whether it moves items or bytes",
                    "C's trap is `p + 1` moving a whole item silently, so the unit is always written (D316)",
                )
                .fix("offset.items"));
            }
            _ => {
                return Err(Report::refused(
                    argument.value.span,
                    "`.%moved` takes `offset.items:` or `offset.bytes:`",
                    "(D316)",
                ));
            }
        };
        let (count, found) = self.check_expression(
            &argument.value,
            Some(&Type::plain_integer(crate::types::IntegerKind::I64)),
        )?;
        if !matches!(found, Type::Integer { unit: None, .. }) {
            return Err(Report::refused(
                argument.value.span,
                format!("a move is a plain integer, and this is `{found}`"),
                "(D316)",
            ));
        }
        let bytes = Node {
            span,
            kind: NodeKind::Arithmetic {
                operator: crate::ir::Arithmetic::Wrap(crate::ast::WordOperator::Times),
                number: crate::ir::Number::Integer(crate::types::IntegerKind::I64),
                left: Box::new(count),
                right: Box::new(Node {
                    span,
                    kind: NodeKind::Constant(crate::types::Value::Integer(item_bytes)),
                }),
            },
        };
        Ok(RawCall {
            node: Node {
                span,
                kind: NodeKind::PointerMoved {
                    pointer: Box::new(pointer),
                    bytes: Box::new(bytes),
                },
            },
            value_type: pointer_type.clone(),
            failures: Vec::new(),
            unnamed: None,
        })
    }

    /// The variable `.of` points at, which stays lent while the pointer exists. (D315)
    pub(super) fn pointed_at(value: &Expression) -> Option<String> {
        let ExpressionKind::Pointer { how, arguments, .. } = &value.kind else {
            return None;
        };
        if how.text != "of" {
            return None;
        }
        let ExpressionKind::Hand { value, .. } = &arguments.first()?.value.kind else {
            return None;
        };
        super::body::root_variable(value)
    }
}
