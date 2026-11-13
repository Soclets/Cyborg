//! Generics (D234, D235, D239, D240, D241, D243, D244): requirements, which are interfaces and type sets; `satisfies`
//! lines; and type parameters, which a generic's body is checked against once, then filled in by each call.

use std::collections::{HashMap, HashSet};

use crate::diagnostic::Diagnostic;
use crate::source::Span;
use crate::syntax::{self, ItemKind, Requirement, RequirementEntry};

use super::types::Type;
use super::{Check, Checker};

const MIXED_WHY: &str =
    "an `interface` lists what a type can do, and a `type_set` lists types, and the two are never mixed (D235)";
const SATISFIES_WHY: &str =
    "a type fits an interface only through a `satisfies` line, which the compiler checks there (D235)";
const NAME_WHY: &str = "a type parameter is `?` and a lowercase name, declared in `< >` straight after the name (D234)";
const KIND_WHY: &str =
    "a unit parameter goes only where a unit goes, and a type parameter only where a type goes (D243)";
pub const USES_WHY: &str = "a generic body may use on a `?item` only what its requirement lists, and only an interface gives operations (D235, D240)";

/// An interface: the operators a type must have, each with how many values it takes, and the interfaces it includes.
#[derive(Clone, Debug, Default)]
pub struct Interface {
    pub operators: Vec<(String, usize)>,
    pub includes: Vec<String>,
}

/// What a type parameter needs.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Needs {
    /// A type: the interfaces it satisfies and the type set it's in, if any, as written.
    Type { interfaces: Vec<String>, set: Option<String>, written: String },
    /// A unit: any unit, or one in a family (D243).
    Unit { family: Option<String> },
}

/// A generic's type parameter.
#[derive(Clone, Debug)]
pub struct TypeParameter {
    pub name: String,
    pub needs: Needs,
    pub span: Span,
}

/// What a type parameter stands for while a body is checked: itself, in the generic's own body, or what a call
/// filled it with.
#[derive(Clone, Debug)]
pub enum Bound {
    Itself(Needs),
    Filled(Type),
    Unit(super::types::Unit),
}

/// A generic function: its text, checked again for each copy, its type parameters and its signature over them.
#[derive(Clone)]
pub struct GenericFunction {
    pub syntax: syntax::Function,
    pub parameters: Vec<TypeParameter>,
    pub signature: super::Signature,
}

/// A copy of a generic function a call asked for, waiting to be checked with its types.
#[derive(Clone)]
pub struct Instance {
    pub generic: String,
    pub bounds: Vec<(String, Bound)>,
    pub signature: super::Signature,
    pub call: Span,
}

/// Every requirement, `satisfies` line and generic of the program.
#[derive(Default)]
pub struct Generics {
    pub functions: HashMap<String, GenericFunction>,
    /// Each copy made, by its generic and what fills its type parameters, to its function.
    pub instances: HashMap<(String, Vec<String>), usize>,
    pub waiting: Vec<Instance>,
    /// Whether the body being checked calls a generic.
    pub called: bool,
    pub interfaces: HashMap<String, Interface>,
    pub type_sets: HashMap<String, Vec<Type>>,
    /// Each type, as it's written, with each interface it said it satisfies.
    pub satisfies: HashSet<(String, String)>,
}

impl Generics {
    /// Every operator an interface lists, through what it includes.
    #[must_use]
    pub fn operators(&self, interface: &str) -> Vec<(String, usize)> {
        let mut found = Vec::new();
        let mut seen = HashSet::new();
        let mut waiting = vec![interface.to_string()];
        while let Some(name) = waiting.pop() {
            if !seen.insert(name.clone()) {
                continue;
            }
            if let Some(info) = self.interfaces.get(&name) {
                found.extend(info.operators.iter().cloned());
                waiting.extend(info.includes.iter().cloned());
            }
        }
        found
    }

    /// Whether an interface is, or includes, another.
    #[must_use]
    pub fn includes(&self, interface: &str, wanted: &str) -> bool {
        let mut seen = HashSet::new();
        let mut waiting = vec![interface.to_string()];
        while let Some(name) = waiting.pop() {
            if name == wanted {
                return true;
            }
            if seen.insert(name.clone())
                && let Some(info) = self.interfaces.get(&name)
            {
                waiting.extend(info.includes.iter().cloned());
            }
        }
        false
    }
}

/// The operator an interface must list for a written one: `above`, `at_least` and `at_most` follow from `below`, and
/// `!=` from `==` (D237).
#[must_use]
pub fn needed_operator(written: &str) -> &str {
    match written {
        "above" | "at_least" | "at_most" => "below",
        "!=" => "==",
        other => other,
    }
}

impl Checker<'_> {
    /// Interfaces and type sets, before anything uses them (D235, D238).
    pub fn declare_requirements(&mut self, file: &syntax::File) {
        for item in &file.items {
            match &item.kind {
                ItemKind::Interface(declared) => {
                    let name = declared.name.text();
                    self.check_closer(&declared.end, &format!("end interface {name}"), super::CLOSER_WHY);
                    if let [first, _, ..] = declared.generics.as_slice() {
                        let comma = Span::new(self.file, first.span.end as usize, first.span.end as usize + 1);
                        self.error(Diagnostic::not_designed(
                            comma,
                            "how a type slot is written",
                            "not decided yet (standard 12.5)",
                        ));
                        continue;
                    }
                    let mut interface = Interface::default();
                    for entry in &declared.entries {
                        match entry {
                            RequirementEntry::Includes(path) => interface.includes.push(path.text()),
                            RequirementEntry::Op { operator, values, .. } => {
                                interface.operators.push((operator.text.clone(), *values));
                            }
                            RequirementEntry::Law(_) => {}
                            RequirementEntry::Type(written) => {
                                self.error(Diagnostic::refused(
                                    written.span,
                                    "an interface lists operations, not types",
                                    MIXED_WHY,
                                ));
                            }
                        }
                    }
                    self.generics.interfaces.insert(name, interface);
                }
                ItemKind::TypeSet(declared) => {
                    let name = declared.name.text();
                    self.check_closer(&declared.end, &format!("end type_set {name}"), super::CLOSER_WHY);
                    let mut types = Vec::new();
                    for entry in &declared.entries {
                        match entry {
                            RequirementEntry::Type(written) => {
                                if let Ok(resolved) = self.resolve_type(written) {
                                    types.push(resolved);
                                }
                            }
                            RequirementEntry::Op { span, .. } => {
                                let keyword = Span::new(self.file, span.start as usize, span.start as usize + 2);
                                self.error(Diagnostic::refused(
                                    keyword,
                                    "a type set lists types, not operations",
                                    MIXED_WHY,
                                ));
                            }
                            RequirementEntry::Includes(path) => {
                                self.error(Diagnostic::refused(
                                    path.span,
                                    "a type set lists types, not other requirements",
                                    MIXED_WHY,
                                ));
                            }
                            RequirementEntry::Law(span) => {
                                self.error(Diagnostic::refused(*span, "a type set lists types, not laws", MIXED_WHY));
                            }
                        }
                    }
                    self.generics.type_sets.insert(name, types);
                }
                _ => {}
            }
        }
    }

    /// `reading satisfies ordered;`: the type has every operator the interface lists (D235).
    pub fn check_satisfies(&mut self, file: &syntax::File) {
        for item in &file.items {
            let ItemKind::Satisfies { subject, interface } = &item.kind else { continue };
            let interface_name = interface.text();
            if self.generics.type_sets.contains_key(&interface_name) {
                self.error(Diagnostic::refused(
                    interface.span,
                    format!("`{interface_name}` is a type set, which nothing joins later"),
                    "a type set is a closed list of types, and only an interface is satisfied (D235)",
                ));
                continue;
            }
            if !self.generics.interfaces.contains_key(&interface_name) {
                self.error(Diagnostic::refused(
                    interface.span,
                    format!("`{interface_name}` isn't an interface"),
                    "an interface is declared with `interface` (D235)",
                ));
                continue;
            }
            let written = syntax::Type {
                kind: syntax::TypeKind::Named {
                    path: subject.clone(),
                    fill: Vec::new(),
                    fill_span: None,
                    labels: Vec::new(),
                    labels_span: None,
                },
                span: subject.span,
            };
            let Ok(subject_type) = self.resolve_type(&written) else { continue };
            let shown = subject_type.to_string();
            let mut fits = true;
            for (operator, values) in self.generics.operators(&interface_name) {
                if !self.has_operator(&subject_type, &operator, values) {
                    let operands = vec![format!("`{shown}`"); values].join(" and ");
                    self.error(Diagnostic::refused(
                        item.span,
                        format!("`{shown}` has no `op {operator}` for {operands}, which `{interface_name}` lists"),
                        SATISFIES_WHY,
                    ));
                    fits = false;
                    break;
                }
            }
            if fits {
                self.generics.satisfies.insert((shown, interface_name));
            }
        }
    }

    /// Whether a type has an operator: a built-in type its own, and a struct or union what it defines (D236, D237).
    fn has_operator(&self, value_type: &Type, operator: &str, values: usize) -> bool {
        if values != 2 {
            return false;
        }
        match value_type {
            Type::Integer { .. } => matches!(
                operator,
                "below" | "==" | "+" | "-" | "x" | "div.trunc" | "div.floor" | "div.exact" | "mod.trunc" | "mod.floor"
            ),
            Type::Float { .. } => matches!(operator, "below" | "==" | "+" | "-" | "x" | "div"),
            Type::Bool => operator == "==",
            Type::Struct { .. } | Type::Union { .. } | Type::New { .. } => {
                let shown = value_type.to_string();
                self.operators.contains_key(&(operator.to_string(), shown.clone(), shown))
            }
            _ => false,
        }
    }

    /// A generic's type parameters, each with what it needs (D234, D239, D243).
    pub fn type_parameters(
        &mut self,
        name: Option<&syntax::Path>,
        generics: &[syntax::Generic],
    ) -> Check<Vec<TypeParameter>> {
        let typed: Vec<&syntax::Generic> =
            generics.iter().filter(|generic| generic.kind == syntax::GenericKind::Type).collect();
        let mut parameters = Vec::new();
        if let (Some(name), Some(first)) = (name, typed.first()) {
            // `<` touches the name for type parameters (D234).
            let opener = first.span.start.saturating_sub(1);
            if name.span.end < opener {
                let span = Span::new(self.file, name.span.start as usize, opener as usize + 1);
                return Err(self.error(Diagnostic::refused_fix(span, format!("{}<", name.text()), NAME_WHY)));
            }
        }
        for generic in typed {
            let shown = format!("?{}", generic.name.text);
            if generic.name.text.chars().any(char::is_uppercase) {
                return Err(self.error(Diagnostic::refused_fix(
                    generic.name.span,
                    format!("?{}", generic.name.text.to_lowercase()),
                    NAME_WHY,
                )));
            }
            let Some(requirement) = &generic.requirement else {
                return Err(self.error(Diagnostic::refused_fix(
                    generic.name.span,
                    format!("{shown}: any.type"),
                    "every type parameter writes what it needs, `any.type` when it needs nothing (D239, D243)",
                )));
            };
            let mut paths = Vec::new();
            flatten(requirement, &mut paths);
            let needs = self.needs(&shown, generic.name.span, &paths)?;
            parameters.push(TypeParameter { name: generic.name.text.clone(), needs, span: generic.name.span });
        }
        Ok(parameters)
    }

    fn needs(&mut self, shown: &str, span: Span, paths: &[&syntax::Path]) -> Check<Needs> {
        let mut interfaces = Vec::new();
        let mut set = None;
        let mut unit = None;
        let mut other = false;
        for path in paths {
            let text = path.text();
            match text.as_str() {
                "any" => {
                    return Err(self.error(Diagnostic::refused(
                        path.span,
                        "`any` says which kind it is",
                        "a type parameter that needs nothing is `any.type`, and a unit parameter `any.unit`, so a bare `any` would hide which (D243)",
                    )));
                }
                "any.type" => other = true,
                "any.unit" => unit = Some(None),
                _ if text.starts_with("std::") => {
                    return Err(self.error(Diagnostic::not_designed(
                        path.span,
                        "which interfaces `std` declares for its own types, and their names",
                        "not decided yet (standard 12.5)",
                    )));
                }
                _ if self.generics.interfaces.contains_key(&text) => {
                    interfaces.push(text);
                    other = true;
                }
                _ if self.generics.type_sets.contains_key(&text) => {
                    if set.is_some() {
                        return Err(self.not_yet(path.span, "two type sets in one requirement".to_string()));
                    }
                    set = Some(text);
                    other = true;
                }
                _ if self.units.is_family(&text) => unit = Some(Some(text)),
                _ => {
                    return Err(self.error(Diagnostic::refused(
                        path.span,
                        format!("`{text}` isn't an interface, a type set or a unit family"),
                        "a requirement is an interface, a type set, `any.type`, `any.unit` or a unit family (D235, D243)",
                    )));
                }
            }
        }
        if let Some(family) = unit {
            if other || paths.len() > 1 {
                return Err(self.error(Diagnostic::refused(
                    span,
                    format!("`{shown}` mixes a unit with something else"),
                    "a unit parameter needs `any.unit` or one family, and nothing else (D243)",
                )));
            }
            return Ok(Needs::Unit { family });
        }
        let written = paths.iter().map(|path| path.text()).collect::<Vec<_>>().join(" and ");
        Ok(Needs::Type { interfaces, set, written })
    }

    /// What `?name` stands for here, if it's in scope.
    #[must_use]
    pub fn bound(&self, name: &str) -> Option<&Bound> {
        self.type_scope.iter().rev().find(|(each, _)| each == name).map(|(_, bound)| bound)
    }

    /// `?item` written where a whole type goes.
    pub fn parameter_type(&mut self, name: &syntax::Name) -> Check<Type> {
        match self.bound(&name.text).cloned() {
            Some(Bound::Itself(Needs::Type { written, .. })) => {
                Ok(Type::Param { name: name.text.clone(), needs: written })
            }
            Some(Bound::Filled(filled)) => Ok(filled),
            Some(Bound::Itself(Needs::Unit { .. }) | Bound::Unit(_)) => Err(self.error(Diagnostic::refused(
                name.span,
                format!(
                    "`?{}` is a unit parameter, so it goes after a number type, as in `f64.?{}`",
                    name.text, name.text
                ),
                KIND_WHY,
            ))),
            None => Err(self.error(Diagnostic::refused(
                name.span,
                format!("`?{}` isn't a type parameter here", name.text),
                "a type parameter is declared in `< >` straight after a generic's name (D234)",
            ))),
        }
    }

    /// `?u` written where a unit goes.
    pub fn parameter_unit(&mut self, name: &str, span: Span) -> Check<super::types::Unit> {
        match self.bound(name).cloned() {
            Some(Bound::Itself(Needs::Unit { .. })) => Ok(super::types::Unit::named(&format!("?{name}"))),
            Some(Bound::Unit(unit)) => Ok(unit),
            Some(Bound::Itself(Needs::Type { .. }) | Bound::Filled(_)) => Err(self.error(Diagnostic::refused(
                span,
                format!("`?{name}` is a type parameter, so it goes where a whole type goes"),
                KIND_WHY,
            ))),
            None => Err(self.error(Diagnostic::refused(
                span,
                format!("`?{name}` isn't a unit parameter here"),
                "a unit parameter is declared in `< >` straight after a generic's name (D243)",
            ))),
        }
    }

    /// An operator on a type parameter in a generic's own body, which its requirement has to list (D235, D240).
    pub fn parameter_operator(&mut self, span: Span, written: &str, parameter: &Type) -> Check<()> {
        let Type::Param { name, needs } = parameter else { return Ok(()) };
        let operator = needed_operator(written);
        let listed = match self.bound(name) {
            Some(Bound::Itself(Needs::Type { interfaces, .. })) => interfaces
                .iter()
                .any(|interface| self.generics.operators(interface).iter().any(|(each, _)| each == operator)),
            _ => false,
        };
        if listed {
            return Ok(());
        }
        Err(self.error(Diagnostic::refused(
            span,
            format!("`?{name}` needs `{needs}`, which doesn't list `op {operator}`"),
            USES_WHY,
        )))
    }
}

fn flatten<'requirement>(requirement: &'requirement Requirement, paths: &mut Vec<&'requirement syntax::Path>) {
    match requirement {
        Requirement::Path(path) => paths.push(path),
        Requirement::And(left, right) => {
            flatten(left, paths);
            flatten(right, paths);
        }
        Requirement::LastsAtLeast(_) => {}
    }
}

impl Checker<'_> {
    /// A call to a generic function: each type parameter is filled from the arguments, or from the type the result
    /// goes into for bare literals, what it needs is checked, and the call goes to the copy for those types (D235,
    /// D241). Inside another generic's own body, it goes to the generic itself, since that body is only checked.
    pub fn generic_signature(
        &mut self,
        body: &super::Body,
        call: Span,
        name: &str,
        arguments: &[syntax::Argument],
        expected: Option<Type>,
    ) -> Check<super::Signature> {
        let Some(generic) = self.generics.functions.get(name).cloned() else { return Err(super::Reported) };
        self.generics.called = true;
        if generic.signature.result.is_some() {
            return Err(self.not_yet(call, "a generic function that can fail".to_string()));
        }
        let matched = self.match_arguments(call, &generic.signature, arguments)?;
        let mut filled: Vec<(String, Type)> = Vec::new();
        for (each, (_, parameter_type, _)) in matched.iter().zip(&generic.signature.parameters) {
            let super::contracts::Matched::Argument(argument) = each else { continue };
            if !mentions_parameter(parameter_type) {
                continue;
            }
            let Some(actual) = self.argument_type(body, &argument.value) else {
                if is_literal(&argument.value) {
                    continue;
                }
                return Err(self.not_yet(argument.value.span, "this argument to a generic".to_string()));
            };
            self.unify(parameter_type, &actual, argument.span, &mut filled)?;
        }
        // Bare literals take the type the result goes into (D241).
        if let (Type::Param { name: returned, .. }, Some(expected)) = (&generic.signature.returns, expected)
            && !filled.iter().any(|(each, _)| each == returned)
        {
            filled.push((returned.clone(), expected));
        }
        for parameter in &generic.parameters {
            let Some((_, bound)) = filled.iter().find(|(each, _)| *each == parameter.name).cloned() else {
                return Err(self.error(Diagnostic::refused(
                    call,
                    format!(
                        "nothing here says which type `?{}` is, so this line says one, as in `: i32 =`",
                        parameter.name
                    ),
                    "a call fills a type parameter from its arguments, and bare literals from the type the result goes into (D241, D50)",
                )));
            };
            self.meets(call, &parameter.needs, &bound)?;
        }
        let substitute = |value_type: &Type| substituted(value_type, &filled);
        let mut signature = generic.signature.clone();
        signature.parameters = signature
            .parameters
            .iter()
            .map(|(each, value_type, ownership)| (each.clone(), substitute(value_type), *ownership))
            .collect();
        signature.returns = substitute(&signature.returns);
        // Inside a generic's own body, nothing is built, so the call stays on the generic.
        if filled.iter().any(|(_, value_type)| mentions_parameter(value_type)) {
            return Ok(signature);
        }
        let shown: Vec<String> = filled.iter().map(|(_, value_type)| value_type.to_string()).collect();
        let key = (name.to_string(), shown.clone());
        if let Some(id) = self.generics.instances.get(&key) {
            signature.id = *id;
            return Ok(signature);
        }
        let id = self.program.functions.len();
        self.program.functions.push(crate::checked::Function {
            symbol: format!("cyborg.{name}<{}>", shown.join(", ")),
            parameters: Vec::new(),
            returns: signature.returns.machine(),
            locals: Vec::new(),
            body: Vec::new(),
            sites: Vec::new(),
            template: false,
        });
        signature.id = id;
        self.generics.instances.insert(key, id);
        let bounds = filled.into_iter().map(|(each, value_type)| (each, Bound::Filled(value_type))).collect();
        self.generics.waiting.push(Instance { generic: name.to_string(), bounds, signature: signature.clone(), call });
        Ok(signature)
    }

    /// A value's type, worked out without checking it, so it isn't checked twice.
    fn argument_type(&self, body: &super::Body, value: &syntax::Expression) -> Option<Type> {
        self.expression_type_quietly(body, value).map(|value_type| value_type.plain())
    }

    /// Fills type parameters by matching a parameter's type against an argument's.
    fn unify(&mut self, pattern: &Type, actual: &Type, span: Span, filled: &mut Vec<(String, Type)>) -> Check<()> {
        match (pattern, actual) {
            (Type::Param { name, .. }, _) => {
                if let Some((_, earlier)) = filled.iter().find(|(each, _)| each == name) {
                    if earlier != actual {
                        return Err(self.error(Diagnostic::refused(
                            span,
                            format!("`?{name}` is `{earlier}` from an earlier argument, and this is `{actual}`"),
                            "a call fills each type parameter with one type (D241)",
                        )));
                    }
                    return Ok(());
                }
                filled.push((name.clone(), actual.clone()));
                Ok(())
            }
            (Type::List { item, .. }, Type::List { item: other, .. })
            | (Type::Array { item, .. }, Type::Array { item: other, .. }) => self.unify(item, other, span, filled),
            _ => Ok(()),
        }
    }

    /// Whether a type fills a type parameter: it's in the type set, and has said it satisfies each interface (D235).
    fn meets(&mut self, call: Span, needs: &Needs, value_type: &Type) -> Check<()> {
        let Needs::Type { interfaces, set, .. } = needs else {
            return Err(self.not_yet(call, "a unit parameter filled by a call".to_string()));
        };
        // Another generic's own type parameter fills it when its requirement says as much.
        if let Type::Param { name, needs: written } = value_type {
            let theirs = match self.bound(name) {
                Some(Bound::Itself(Needs::Type { interfaces, set, .. })) => (interfaces.clone(), set.clone()),
                _ => (Vec::new(), None),
            };
            let missing = interfaces
                .iter()
                .find(|wanted| !theirs.0.iter().any(|interface| self.generics.includes(interface, wanted)))
                .cloned()
                .or_else(|| set.clone().filter(|wanted| theirs.1.as_ref() != Some(wanted)));
            if let Some(missing) = missing {
                return Err(self.error(Diagnostic::refused(
                    call,
                    format!("`?{name}` needs `{written}`, which doesn't give `{missing}`"),
                    USES_WHY,
                )));
            }
            return Ok(());
        }
        let shown = value_type.to_string();
        if let Some(set) = set {
            let members = self.generics.type_sets.get(set).cloned().unwrap_or_default();
            if !members.iter().any(|member| member.to_string() == shown) {
                return Err(self.error(Diagnostic::refused(
                    call,
                    format!("`{shown}` isn't in the type set `{set}`"),
                    "a type set is a closed list of the types it allows (D235)",
                )));
            }
        }
        for interface in interfaces {
            let said = self
                .generics
                .satisfies
                .iter()
                .any(|(each, satisfied)| *each == shown && self.generics.includes(satisfied, interface));
            if !said {
                return Err(self.error(Diagnostic::refused(
                    call,
                    format!("`{shown}` hasn't said it satisfies `{interface}`"),
                    SATISFIES_WHY,
                )));
            }
        }
        Ok(())
    }

    /// Checks each copy of a generic a call asked for, with its types filled in (D235).
    pub fn build_instances(&mut self) {
        while let Some(instance) = self.generics.waiting.pop() {
            let Some(generic) = self.generics.functions.get(&instance.generic).cloned() else { continue };
            let outer = self.type_scope.len();
            self.type_scope.extend(instance.bounds.clone());
            let was = std::mem::replace(&mut self.instance, true);
            let before = self.errors.len();
            self.function_body(&generic.syntax, &instance.signature);
            // The generic's own body was checked against its requirements, so a copy shouldn't find anything new;
            // if it does, that's this compiler's gap, said once where the copy was asked for.
            if self.errors.len() > before {
                self.errors.truncate(before);
                let what = format!("this call to `{}` with these types", instance.generic);
                self.not_yet(instance.call, what);
            }
            self.instance = was;
            self.type_scope.truncate(outer);
        }
    }
}

/// Whether a type has a type parameter in it.
#[must_use]
pub fn mentions_parameter(value_type: &Type) -> bool {
    match value_type {
        Type::Param { .. } => true,
        Type::List { item, .. } | Type::Array { item, .. } => mentions_parameter(item),
        Type::Borrow { inner, .. } | Type::New { inner, .. } => mentions_parameter(inner),
        _ => false,
    }
}

/// A type with its type parameters filled in.
fn substituted(value_type: &Type, filled: &[(String, Type)]) -> Type {
    match value_type {
        Type::Param { name, .. } => {
            filled.iter().find(|(each, _)| each == name).map_or_else(|| value_type.clone(), |(_, found)| found.clone())
        }
        Type::List { item, min, max, len_type } => {
            Type::List { item: Box::new(substituted(item, filled)), min: *min, max: *max, len_type: *len_type }
        }
        Type::Array { item, len, len_type } => {
            Type::Array { item: Box::new(substituted(item, filled)), len: *len, len_type: *len_type }
        }
        Type::Borrow { mutable, held, sources, inner } => Type::Borrow {
            mutable: *mutable,
            held: *held,
            sources: sources.clone(),
            inner: Box::new(substituted(inner, filled)),
        },
        other => other.clone(),
    }
}

/// A bare literal, which has no type of its own (D50).
fn is_literal(value: &syntax::Expression) -> bool {
    match &value.kind {
        syntax::ExpressionKind::Integer(_) | syntax::ExpressionKind::Float(_) => true,
        syntax::ExpressionKind::Unary { operand, .. } => is_literal(operand),
        syntax::ExpressionKind::Group(inner) => is_literal(inner),
        _ => false,
    }
}
