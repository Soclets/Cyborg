//! Naming a type: `type.alias score = i32;` is another name, and `type.new user_id = i64;` a different type over the
//! same data, alone or in a family (D134). Names resolve when first needed, so they work in any order (D120).

use std::collections::HashMap;

use crate::diagnostic::Diagnostic;
use crate::source::Span;
use crate::syntax;

use super::types::Type;
use super::{Check, Checker, NOT_DECIDED, Reported};

/// Why a `type.new` doesn't mix.
pub const NEW_WHY: &str =
    "a `type.new` is a different type, so it doesn't mix with another, or with what it's made from (D134)";
const FAMILY_WHY: &str = "a family's types are always used with the family (D134)";
const KINDS_WHY: &str = "a family doesn't mix `type.alias` and `type.new` (D134)";

#[derive(Clone, Debug)]
enum State {
    Waiting,
    Resolving,
    Done(Type),
    Broken,
}

#[derive(Clone, Debug)]
pub struct TypeName {
    /// `type.new` rather than `type.alias`.
    pub new: bool,
    pub target: syntax::Type,
    pub name_span: Span,
    /// Declared in a `fam.type.…`.
    pub in_family: bool,
    state: State,
}

/// Every type name, by its full name: `score`, `sizes.small`, or `my::str` for one that clashes (D75).
#[derive(Default)]
pub struct TypeNames {
    pub declared: HashMap<String, TypeName>,
    /// In the order written, so errors come out in that order.
    order: Vec<String>,
    /// Each family, with whether it's of `type.new`.
    families: HashMap<String, bool>,
    /// A family member's short name, to each family's full name for it.
    members: HashMap<String, Vec<String>>,
    resolving: Vec<String>,
}

impl Checker<'_> {
    /// Reads every `type.…` and `fam.type.…`, before structs, so either can name the other.
    pub fn declare_type_names(&mut self, file: &syntax::File) {
        let aggregate_names: Vec<String> = file
            .items
            .iter()
            .filter_map(|item| match &item.kind {
                syntax::ItemKind::Struct(declared) => Some(declared.name.text()),
                syntax::ItemKind::Union(declared) => Some(declared.name.text()),
                _ => None,
            })
            .collect();
        for item in &file.items {
            match &item.kind {
                syntax::ItemKind::TypeName { keyword, name, target } => {
                    let new = match keyword.text().as_str() {
                        "type.alias" => false,
                        "type.new" => true,
                        other => {
                            self.error(Diagnostic::not_designed(keyword.span, format!("`{other}`"), NOT_DECIDED));
                            continue;
                        }
                    };
                    let mut text = name.text();
                    if super::clashes(&text) {
                        text = format!("my::{text}");
                    }
                    self.add_type_name(&text, name.span, new, target, false, &aggregate_names);
                }
                syntax::ItemKind::Family(family) => {
                    let keyword = family.keyword.text();
                    let new = match keyword.as_str() {
                        "fam.type.alias" => false,
                        "fam.type.new" => true,
                        _ => continue,
                    };
                    let set = family.name.text();
                    self.check_closer(&family.end, &format!("end {keyword} {set}"), super::CLOSER_WHY);
                    if let Some(&was_new) = self.type_names.families.get(&set) {
                        let place = family.entries.first().map_or(family.name.span, |entry| entry.name.span);
                        if was_new == new {
                            self.error(Diagnostic::not_designed(
                                place,
                                format!("declaring `{set}` twice"),
                                NOT_DECIDED,
                            ));
                        } else {
                            self.error(Diagnostic::refused(
                                place,
                                format!("`{set}` is already declared with the other kind"),
                                KINDS_WHY,
                            ));
                        }
                        continue;
                    }
                    self.type_names.families.insert(set.clone(), new);
                    for entry in &family.entries {
                        let syntax::FamilyEntryKind::TypeSetTo(target) = &entry.kind else {
                            self.error(Diagnostic::refused(
                                entry.span,
                                "a family of types gives each one a type with `=`",
                                "`ids.user = i64;` names a type in the family (D134)",
                            ));
                            continue;
                        };
                        let written = entry.name.text();
                        let Some(short) = written.strip_prefix(&format!("{set}.")) else {
                            self.error(Diagnostic::refused_fix(
                                entry.name.span,
                                format!("{set}.{written}"),
                                FAMILY_WHY,
                            ));
                            continue;
                        };
                        self.type_names.members.entry(short.to_string()).or_default().push(written.clone());
                        self.add_type_name(&written, entry.name.span, new, target, true, &aggregate_names);
                    }
                }
                _ => {}
            }
        }
    }

    fn add_type_name(
        &mut self,
        name: &str,
        span: Span,
        new: bool,
        target: &syntax::Type,
        in_family: bool,
        aggregate_names: &[String],
    ) {
        if self.type_names.declared.contains_key(name) || aggregate_names.iter().any(|each| each == name) {
            self.error(Diagnostic::not_designed(span, format!("declaring `{name}` twice"), NOT_DECIDED));
            return;
        }
        if new && mentions(target, name) {
            self.error(Diagnostic::not_designed(
                span,
                format!("a `type.new` that holds itself, as `{name}` does"),
                NOT_DECIDED,
            ));
            return;
        }
        self.type_names.order.push(name.to_string());
        self.type_names.declared.insert(
            name.to_string(),
            TypeName { new, target: target.clone(), name_span: span, in_family, state: State::Waiting },
        );
    }

    /// Resolves every name, so each one's errors are said once, where it's declared. An alias of a struct or union
    /// can build one and name its cases, so it joins their names.
    pub fn resolve_type_names(&mut self) {
        for name in self.type_names.order.clone() {
            if let Ok(resolved) = self.resolve_type_name(&name) {
                let alias = self.type_names.declared.get(&name).is_some_and(|each| !each.new);
                if alias && matches!(resolved, Type::Struct { .. } | Type::Union { .. }) {
                    self.aggregates.names.insert(name, resolved);
                }
            }
        }
    }

    fn resolve_type_name(&mut self, name: &str) -> Check<Type> {
        let Some(info) = self.type_names.declared.get(name).cloned() else { return Err(Reported) };
        match info.state {
            State::Done(resolved) => return Ok(resolved),
            State::Broken => return Err(Reported),
            State::Resolving => {
                // A loop: it's said at the name it started from, and every name in it is broken.
                let start = self.type_names.resolving.iter().position(|each| each == name).unwrap_or(0);
                for each in self.type_names.resolving[start..].to_vec() {
                    self.set_state(&each, State::Broken);
                }
                self.error(Diagnostic::refused(
                    info.name_span,
                    format!("`{name}` leads back to itself"),
                    "`type.alias` gives a type another name, so it has to end at a type (D134)",
                ));
                return Err(Reported);
            }
            State::Waiting => {}
        }
        self.set_state(name, State::Resolving);
        self.type_names.resolving.push(name.to_string());
        let result = self.resolve_type(&info.target);
        self.type_names.resolving.pop();
        if matches!(self.type_names.declared.get(name).map(|each| &each.state), Some(State::Broken)) {
            return Err(Reported);
        }
        let resolved = match result {
            Ok(inner) if info.new => Type::New { name: name.to_string(), inner: Box::new(inner) },
            Ok(resolved) => resolved,
            Err(Reported) => {
                self.set_state(name, State::Broken);
                return Err(Reported);
            }
        };
        self.set_state(name, State::Done(resolved.clone()));
        Ok(resolved)
    }

    fn set_state(&mut self, name: &str, state: State) {
        if let Some(info) = self.type_names.declared.get_mut(name) {
            info.state = state;
        }
    }

    /// A written type that names a `type.…`, or a narrowing on one, or a family's member without its family.
    pub fn named_type(&mut self, written: &syntax::Type, text: &str) -> Option<Check<Type>> {
        let syntax::TypeKind::Named { path, fill, labels, .. } = &written.kind else { return None };
        if self.type_names.declared.contains_key(text) {
            if !fill.is_empty() || !labels.is_empty() {
                let what = format!("filling in or describing `{text}`");
                return Some(Err(self.error(Diagnostic::not_designed(
                    written.span,
                    what,
                    "not decided yet (standard 12.5)",
                ))));
            }
            return Some(self.resolve_type_name(text));
        }
        let mut cut = 0;
        while let Some(dot) = text[cut..].find('.') {
            let prefix = &text[..cut + dot];
            if self.type_names.declared.contains_key(prefix) {
                return Some(Err(self.error(Diagnostic::not_designed(
                    path.span,
                    format!("a narrowing on `{prefix}`, a name from `type`"),
                    NOT_DECIDED,
                ))));
            }
            cut += dot + 1;
        }
        None
    }

    /// A family member written without its family, for when nothing else has the name.
    pub fn bare_member(&mut self, path: &syntax::Path, text: &str) -> Option<Reported> {
        let families = self.type_names.members.get(text)?.clone();
        Some(match families.as_slice() {
            [one] => self.error(Diagnostic::refused_fix(path.span, one.clone(), FAMILY_WHY)),
            _ => self.error(Diagnostic::refused(
                path.span,
                format!("`{text}` is in a family, so it's used with it"),
                FAMILY_WHY,
            )),
        })
    }

    /// A method's owner that's a type name: an alias or a family's member isn't decided, and a `type.new` is fine.
    pub fn method_owner(&mut self, owner: &str, span: Span) -> Option<Check<Type>> {
        let info = self.type_names.declared.get(owner)?.clone();
        if info.in_family {
            // Said at the type, before the method's own name.
            let owner_span = Span::new(span.file, span.start as usize, span.start as usize + owner.len());
            return Some(Err(self.error(Diagnostic::not_designed(
                owner_span,
                "a method on a type from a family",
                NOT_DECIDED,
            ))));
        }
        let resolved = match self.resolve_type_name(owner) {
            Ok(resolved) => resolved,
            Err(reported) => return Some(Err(reported)),
        };
        if !info.new {
            let what = format!("a method on `{owner}`, which is another name for `{resolved}`");
            return Some(Err(self.error(Diagnostic::not_designed(span, what, NOT_DECIDED))));
        }
        Some(Ok(resolved))
    }
}

impl Checker<'_> {
    /// Maths or a comparison with a `type.new` in it: on two of the same one, or one and a literal, it isn't
    /// decided, and with anything else it doesn't mix (D134).
    pub fn new_in_operation(
        &mut self,
        span: Span,
        left: &super::expressions::Typed,
        right: &super::expressions::Typed,
        comparing: bool,
    ) -> Option<Reported> {
        let is_new = |value_type: &Type| matches!(value_type, Type::New { .. });
        let literal = |typed: &super::expressions::Typed| typed.literal.is_some() || typed.float_literal.is_some();
        let (new, other) = match (is_new(&left.value_type), is_new(&right.value_type)) {
            (false, false) => return None,
            (true, _) => (left, right),
            (false, true) => (right, left),
        };
        if new.value_type == other.value_type || literal(other) {
            let what = if comparing {
                format!("comparing `{}` values", new.value_type)
            } else {
                "maths on a `type.new`".to_string()
            };
            let why = if comparing { NOT_DECIDED } else { "not decided yet (standard 12.1)" };
            return Some(self.error(Diagnostic::not_designed(span, what, why)));
        }
        Some(self.error(Diagnostic::refused(
            span,
            format!("this is `{}`, where `{}` is needed", other.value_type, new.value_type),
            NEW_WHY,
        )))
    }
}

/// Whether a written type names `name` anywhere inside it.
fn mentions(written: &syntax::Type, name: &str) -> bool {
    match &written.kind {
        syntax::TypeKind::Named { path, fill, .. } => {
            path.text() == name
                || fill.iter().any(|argument| match &argument.value {
                    syntax::TypeArgumentValue::Type(inner) => mentions(inner, name),
                    syntax::TypeArgumentValue::Value(_) => false,
                })
        }
        _ => false,
    }
}
