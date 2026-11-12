//! Generics: type parameters, the interfaces and type sets they need, and filling them in at a
//! call. A generic is checked once, at its definition, where a `?item` can only do what its
//! requirement lists. A call fills each `?item` from its arguments, checks the type fits, and
//! passes a table of how that type does each operation, so the body runs unchanged.
//! (D234, D235, D238, D239, D240, D241)

use std::collections::{HashMap, HashSet};

use super::{Checker, Signature};
use crate::ast::{
    CompareOperator, Expression, ExpressionKind, InterfaceDeclaration, Item, Satisfies,
    TypeParameter, TypeSetDeclaration,
};
use crate::ir::{Arithmetic, Implementation, Node, NodeKind, Number, Table};
use crate::report::{Outcome, Report, Span};
use crate::types::Type;

/// An interface: what it includes, and the operations a type needs for it, written in terms
/// of `?self`. (D235, D238)
#[derive(Clone, Debug)]
pub(super) struct InterfaceInfo {
    own: String,
    includes: Vec<String>,
    operators: Vec<InterfaceOperatorInfo>,
}

#[derive(Clone, Debug)]
pub(super) struct InterfaceOperatorInfo {
    pub(super) operator: String,
    pub(super) left: Type,
    pub(super) right: Type,
    pub(super) result: Type,
}

const WHY_SATISFIES: &str =
    "a type fits an interface only through a `satisfies` line, which the compiler checks there (D235)";
const WHY_USES: &str =
    "a generic body may use on a `?item` only what its requirement lists, and only an interface gives operations (D235, D240)";

/// Puts `with` wherever `own` is, as an interface's `?self` becomes the type that fits it.
fn put(found: &Type, own: &str, with: &Type) -> Type {
    match found {
        Type::Parameter(name) if name == own => with.clone(),
        other => other.clone(),
    }
}

/// A type without its bound, which is what fills a type parameter.
pub(super) fn unbounded(found: &Type) -> Type {
    match found {
        Type::Integer { kind, unit, .. } => Type::Integer {
            kind: *kind,
            bound: None,
            unit: unit.clone(),
        },
        Type::Float {
            kind, unit, rules, ..
        } => Type::Float {
            kind: *kind,
            bound: None,
            unit: unit.clone(),
            rules: *rules,
        },
        other => other.clone(),
    }
}

/// How a built-in type does `operator` on two of itself, giving `result`, if it can.
fn builtin(operator: &str, own: &Type, result: &Type) -> Option<Implementation> {
    let number = match own {
        Type::Integer {
            kind, bound: None, ..
        } => Some(Number::Integer(*kind)),
        Type::Float {
            kind,
            bound: None,
            rules,
            ..
        } => Some(Number::Float(*kind, *rules)),
        _ => None,
    };
    let unitless = matches!(
        own,
        Type::Integer { unit: None, .. } | Type::Float { unit: None, .. }
    );
    match operator {
        "==" if *result == Type::Bool && (number.is_some() || *own == Type::Bool) => {
            Some(Implementation::Compare(CompareOperator::Equal))
        }
        "below" if *result == Type::Bool && number.is_some() => {
            Some(Implementation::Compare(CompareOperator::Below))
        }
        "+" | "-" if result == own => Some(Implementation::Arithmetic(
            if operator == "+" {
                Arithmetic::Add
            } else {
                Arithmetic::Subtract
            },
            number?,
        )),
        "x" if result == own && unitless => {
            Some(Implementation::Arithmetic(Arithmetic::Multiply, number?))
        }
        "div.trunc" | "div.floor" | "div.exact"
            if result == own && unitless && matches!(own, Type::Integer { .. }) =>
        {
            let narrowing = match operator {
                "div.trunc" => crate::ast::Narrowing::Trunc,
                "div.floor" => crate::ast::Narrowing::Floor,
                _ => crate::ast::Narrowing::Exact,
            };
            Some(Implementation::Arithmetic(
                Arithmetic::Divide(narrowing),
                number?,
            ))
        }
        _ => None,
    }
}

impl Checker<'_> {
    /// Declares every interface and type set, before the functions that need them.
    /// trace: D235, D238
    pub(super) fn declare_requirements(&mut self, items: &[&Item]) -> Outcome<()> {
        for item in items {
            let name = match item {
                Item::Interface(interface) => &interface.name,
                Item::TypeSet(set) => &set.name,
                _ => continue,
            };
            if self.interfaces.contains_key(&name.text)
                || self.type_sets.contains_key(&name.text)
                || self.structs.contains_key(&name.text)
                || self.unions.contains_key(&name.text)
            {
                return Err(Report::not_designed(
                    name.span,
                    format!("declaring `{}` twice", name.text),
                    "",
                ));
            }
            // Its name is known before its lines are read, so interfaces can include each other.
            match item {
                Item::Interface(interface) => {
                    self.interfaces.insert(
                        name.text.clone(),
                        InterfaceInfo {
                            own: interface.own.text.clone(),
                            includes: Vec::new(),
                            operators: Vec::new(),
                        },
                    );
                }
                _ => {
                    self.type_sets.insert(name.text.clone(), Vec::new());
                }
            }
        }
        for item in items {
            match item {
                Item::Interface(interface) => self.declare_interface(interface)?,
                Item::TypeSet(set) => self.declare_type_set(set)?,
                _ => {}
            }
        }
        Ok(())
    }

    fn declare_interface(&mut self, interface: &InterfaceDeclaration) -> Outcome<()> {
        let own = interface.own.text.clone();
        let mut includes = Vec::new();
        for included in &interface.includes {
            if self.type_sets.contains_key(&included.text) {
                return Err(Report::refused(
                    included.span,
                    format!("`{}` is a type set, which an interface can't include", included.text),
                    "an `interface` lists what a type can do, and a `type_set` lists types, and the two are never mixed (D235)",
                ));
            }
            if !self.interfaces.contains_key(&included.text) {
                return Err(Report::refused(
                    included.span,
                    format!("there's no interface `{}`", included.text),
                    "an interface includes another interface (D235)",
                ));
            }
            includes.push(included.text.clone());
        }
        self.generic = vec![(own.clone(), Vec::new())];
        let mut operators = Vec::new();
        for operator in &interface.operators {
            let [left, right] = operator.parameters.as_slice() else {
                self.generic.clear();
                return Err(Report::not_designed(
                    operator.span,
                    format!(
                        "an interface's `op {}` that doesn't take two values",
                        operator.operator
                    ),
                    "12.3",
                ));
            };
            let resolved = (|| -> Outcome<InterfaceOperatorInfo> {
                Ok(InterfaceOperatorInfo {
                    operator: operator.operator.clone(),
                    left: self.resolve_type(&left.written_type)?,
                    right: self.resolve_type(&right.written_type)?,
                    result: self.resolve_type(&operator.return_type)?,
                })
            })();
            match resolved {
                Ok(info) => operators.push(info),
                Err(report) => {
                    self.generic.clear();
                    return Err(report);
                }
            }
        }
        self.generic.clear();
        if let Some(info) = self.interfaces.get_mut(&interface.name.text) {
            info.includes = includes;
            info.operators = operators;
        }
        Ok(())
    }

    fn declare_type_set(&mut self, set: &TypeSetDeclaration) -> Outcome<()> {
        let mut members = Vec::new();
        for member in &set.members {
            members.push(self.resolve_type(member)?);
        }
        self.type_sets.insert(set.name.text.clone(), members);
        Ok(())
    }

    /// Every operation an interface needs, its included ones' too, in terms of its `?self`.
    fn operations_of(&self, interface: &str) -> Vec<InterfaceOperatorInfo> {
        let mut seen = HashSet::new();
        let mut waiting = vec![interface.to_string()];
        let mut operations = Vec::new();
        while let Some(name) = waiting.pop() {
            if !seen.insert(name.clone()) {
                continue;
            }
            let Some(info) = self.interfaces.get(&name) else {
                continue;
            };
            let own = Type::Parameter(info.own.clone());
            for operation in &info.operators {
                operations.push(InterfaceOperatorInfo {
                    operator: operation.operator.clone(),
                    left: put(&operation.left, &info.own, &own),
                    right: put(&operation.right, &info.own, &own),
                    result: put(&operation.result, &info.own, &own),
                });
            }
            waiting.extend(info.includes.iter().cloned());
        }
        operations
    }

    /// The interface `interface` is, or includes, `wanted`.
    fn includes(&self, interface: &str, wanted: &str) -> bool {
        let mut seen = HashSet::new();
        let mut waiting = vec![interface.to_string()];
        while let Some(name) = waiting.pop() {
            if name == wanted {
                return true;
            }
            if seen.insert(name.clone()) {
                if let Some(info) = self.interfaces.get(&name) {
                    waiting.extend(info.includes.iter().cloned());
                }
            }
        }
        false
    }

    /// How `own` does an interface's operation, written in terms of its `?self`: the type's
    /// own `op`, or a built-in one.
    fn implementation(
        &self,
        operation: &InterfaceOperatorInfo,
        own: &Type,
    ) -> Option<Implementation> {
        let self_name = |found: &Type| match found {
            Type::Parameter(name) => Some(name.clone()),
            _ => None,
        };
        let own_name = self_name(&operation.left)
            .or_else(|| self_name(&operation.right))
            .or_else(|| self_name(&operation.result))?;
        let left = put(&operation.left, &own_name, own);
        let right = put(&operation.right, &own_name, own);
        let result = put(&operation.result, &own_name, own);
        if let Some(found) = self.operators.iter().find(|found| {
            found.operator == operation.operator
                && found.left == left
                && found.right == right
                && found.result == result
        }) {
            return self
                .signatures
                .get(&found.function)
                .map(|signature| Implementation::Function(signature.index));
        }
        if left == *own && right == *own {
            return builtin(&operation.operator, own, &result);
        }
        None
    }

    /// Checks each `reading satisfies ordered;` line: the interface is one, and the type has
    /// every operation it lists. Only the type's package or the interface's can write it, and
    /// a program is one package, so here it always is. (D235)
    /// trace: D235
    pub(super) fn declare_satisfies(&mut self, items: &[&Item]) -> Outcome<()> {
        for item in items {
            let Item::Satisfies(line) = item else {
                continue;
            };
            self.declare_one_satisfies(line)?;
        }
        Ok(())
    }

    fn declare_one_satisfies(&mut self, line: &Satisfies) -> Outcome<()> {
        let interface = &line.interface;
        if self.type_sets.contains_key(&interface.text) {
            return Err(Report::refused(
                interface.span,
                format!(
                    "`{}` is a type set, which nothing joins later",
                    interface.text
                ),
                "a type set is a closed list of types, and only an interface is satisfied (D235)",
            ));
        }
        if !self.interfaces.contains_key(&interface.text) {
            return Err(Report::refused(
                interface.span,
                format!("there's no interface `{}`", interface.text),
                WHY_SATISFIES,
            ));
        }
        let own = self.resolve_type(&line.written_type)?;
        if self.is_must_give(&own) {
            return Err(Report::not_designed(
                line.written_type.span,
                "a `must_Give` type fitting an interface",
                "",
            ));
        }
        for operation in self.operations_of(&interface.text) {
            if self.implementation(&operation, &own).is_none() {
                let own_name = Self::own_name(&operation);
                return Err(Report::refused(
                    line.span,
                    format!(
                        "`{own}` has no `op {}` for `{}` and `{}`, which `{}` lists",
                        operation.operator,
                        put(&operation.left, &own_name, &own),
                        put(&operation.right, &own_name, &own),
                        interface.text
                    ),
                    WHY_SATISFIES,
                ));
            }
        }
        self.satisfied.push((own, interface.text.clone()));
        Ok(())
    }

    fn own_name(operation: &InterfaceOperatorInfo) -> String {
        [&operation.left, &operation.right, &operation.result]
            .iter()
            .find_map(|found| match found {
                Type::Parameter(name) => Some(name.clone()),
                _ => None,
            })
            .unwrap_or_default()
    }

    /// Reads a function's or struct's type parameters: each names what it needs, every part
    /// of which is `any`, an interface or a type set. (D235, D239, D240)
    /// trace: D235, D239, D240
    pub(super) fn read_type_parameters(
        &self,
        parameters: &[TypeParameter],
    ) -> Outcome<Vec<(String, Vec<String>)>> {
        let mut read = Vec::new();
        for parameter in parameters {
            if read
                .iter()
                .any(|(name, _): &(String, Vec<String>)| *name == parameter.name.text)
            {
                return Err(Report::refused(
                    parameter.name.span,
                    format!("`?{}` is declared twice", parameter.name.text),
                    "a type parameter's name says which one it is (D234)",
                ));
            }
            let mut requirement = Vec::new();
            let families = self.unit_families();
            for part in &parameter.requirement {
                if part.text.contains("::") {
                    return Err(Report::not_designed(
                        part.span,
                        "which interfaces `std` declares for its own types, and their names",
                        "12.5",
                    ));
                }
                // trace: D243
                if part.text == "any" {
                    return Err(Report::refused(
                        part.span,
                        "`any` says which kind it is",
                        "a type parameter that needs nothing is `any.type`, and a unit parameter `any.unit`, so a bare `any` would hide which (D243)",
                    ));
                }
                if part.text != "any.type"
                    && part.text != "any.unit"
                    && !families.contains(&part.text)
                    && !self.interfaces.contains_key(&part.text)
                    && !self.type_sets.contains_key(&part.text)
                {
                    return Err(Report::refused(
                        part.span,
                        format!("there's no interface, type set or unit family `{}`", part.text),
                        "a type parameter needs `any.type`, an interface or a type set, and a unit parameter `any.unit` or a family (D235, D243)",
                    ));
                }
                requirement.push(part.text.clone());
            }
            let units = requirement
                .iter()
                .filter(|part| *part == "any.unit" || families.contains(*part))
                .count();
            if units > 0 && requirement.len() > 1 {
                return Err(Report::refused(
                    parameter.name.span,
                    format!(
                        "`?{}` mixes a unit with something else",
                        parameter.name.text
                    ),
                    "a unit parameter needs `any.unit` or one family, and nothing else (D243)",
                ));
            }
            read.push((parameter.name.text.clone(), requirement));
        }
        Ok(read)
    }

    /// Whether `found` meets every part of `requirement`. A type parameter of the generic
    /// being checked meets what its own requirement says. (D235, D239, D240)
    fn meets(&self, found: &Type, requirement: &[String], span: Span) -> Outcome<()> {
        for part in requirement {
            if part == "any.type" {
                continue;
            }
            if let Type::Parameter(name) = found {
                let own = self
                    .generic
                    .iter()
                    .find(|(found, _)| found == name)
                    .map(|(_, own)| own.clone())
                    .unwrap_or_default();
                let fits = own
                    .iter()
                    .any(|have| have == part || self.includes(have, part));
                if !fits {
                    return Err(Report::refused(
                        span,
                        format!("`?{name}` doesn't need `{part}`, which this needs"),
                        WHY_USES,
                    ));
                }
                continue;
            }
            if let Some(members) = self.type_sets.get(part) {
                if !members.contains(found) {
                    return Err(Report::refused(
                        span,
                        format!("`{found}` isn't in the type set `{part}`"),
                        "a type set is a closed list of the types it allows (D235)",
                    ));
                }
                continue;
            }
            let fits = self
                .satisfied
                .iter()
                .any(|(own, interface)| own == found && self.includes(interface, part));
            if !fits {
                return Err(Report::refused(
                    span,
                    format!("`{found}` hasn't said it satisfies `{part}`"),
                    WHY_SATISFIES,
                ));
            }
        }
        Ok(())
    }

    /// The table a call passes for one type parameter: how the type that fills it does each
    /// operation its requirement's interfaces list.
    fn table_for(&self, requirement: &[String], found: &Type) -> Vec<(String, Implementation)> {
        let mut table = Vec::new();
        for part in requirement {
            for operation in self.operations_of(part) {
                if table.iter().any(|(name, _)| *name == operation.operator) {
                    continue;
                }
                let implementation = match found {
                    // A generic calling a generic passes on its own caller's table.
                    Type::Parameter(name) => self
                        .generic
                        .iter()
                        .position(|(found, _)| found == name)
                        .map(|parameter| Implementation::Forward {
                            parameter,
                            operator: operation.operator.clone(),
                        }),
                    _ => self.implementation(&operation, found),
                };
                if let Some(implementation) = implementation {
                    table.push((operation.operator.clone(), implementation));
                }
            }
        }
        table
    }

    /// Fills a generic's type parameters at a call: from the arguments' types, or for bare
    /// literals from the type the result goes into. Each type is checked against its
    /// requirement, and the call carries a table of how each does what's needed. (D241)
    /// trace: D241
    pub(super) fn fill_type_parameters(
        &self,
        signature: &Signature,
        arguments: &[crate::ast::Argument],
        expected: Option<&Type>,
        span: Span,
    ) -> Outcome<(Signature, Vec<Table>)> {
        let mut filled = Filled::default();
        // An argument whose form doesn't say its type, like `17 div.trunc 18`, takes the type
        // the others fill, the way a bare literal does, and matters only if nothing else says.
        let mut unread = None;
        for (index, argument) in arguments.iter().enumerate() {
            let parameter = match &argument.label {
                Some(label) => signature.parameters.iter().find(|parameter| {
                    parameter.label.as_deref() == Some(label.text.as_str())
                        || parameter.name == label.text
                }),
                None => signature.parameters.get(index),
            };
            let Some(parameter) = parameter else {
                continue;
            };
            if !mentions_a_parameter(&parameter.parameter_type) {
                continue;
            }
            let value = match &argument.value.kind {
                ExpressionKind::Hand { value, .. } => value.as_ref(),
                _ => &argument.value,
            };
            if super::expression::is_number_literal(value)
                || matches!(&value.kind, ExpressionKind::Negate(inner) if super::expression::is_number_literal(inner))
            {
                continue;
            }
            let Some(found) = self.type_from_form(value) else {
                unread.get_or_insert(argument.value.span);
                continue;
            };
            filled.unify(&parameter.parameter_type, &found, argument.value.span)?;
        }
        let mut tables = Vec::new();
        for (name, requirement) in &signature.type_parameters {
            if self.requirement_is_unit(requirement) {
                Self::fill_unit(signature, name, requirement, expected, &mut filled, span)?;
                tables.push(Vec::new());
                continue;
            }
            if !filled.types.contains_key(name) {
                match (&signature.return_type, expected) {
                    (Type::Parameter(returned), Some(target)) if returned == name => {
                        filled.types.insert(name.clone(), unbounded(target));
                    }
                    _ => {
                        if let Some(unread) = unread {
                            return Err(Report::not_designed(
                                unread,
                                "working out a type parameter from this argument",
                                "",
                            ));
                        }
                        return Err(Report::refused(
                            span,
                            format!("nothing here says which type `?{name}` is, so this line says one, as in `: i32 =`"),
                            "a call fills a type parameter from its arguments, and bare literals from the type the result goes into (D241, D50)",
                        ));
                    }
                }
            }
            let found = filled.types.get(name).cloned().unwrap_or(Type::Void);
            if self.is_must_give(&found) {
                return Err(Report::not_designed(
                    span,
                    format!("a `must_Give` type, `{found}`, filling `?{name}`"),
                    "",
                ));
            }
            self.meets(&found, requirement, span)?;
            tables.push(self.table_for(requirement, &found));
        }
        let mut concrete = signature.clone();
        for parameter in &mut concrete.parameters {
            parameter.parameter_type = filled.put_into(&parameter.parameter_type);
        }
        concrete.return_type = filled.put_into(&concrete.return_type);
        concrete.type_parameters.clear();
        Ok((concrete, tables))
    }

    /// Whether a requirement makes a unit parameter: `any.unit`, or a family. (D243)
    fn requirement_is_unit(&self, requirement: &[String]) -> bool {
        let families = self.unit_families();
        requirement
            .iter()
            .any(|part| part == "any.unit" || families.contains(part))
    }

    /// A unit parameter's unit, from the arguments or else the result's target, in the family
    /// its requirement names, if it names one. (D243)
    /// trace: D243
    fn fill_unit(
        signature: &Signature,
        name: &str,
        requirement: &[String],
        expected: Option<&Type>,
        filled: &mut Filled,
        span: Span,
    ) -> Outcome<()> {
        if !filled.units.contains_key(name) {
            let wanted = format!("?{name}");
            match (unit_of(&signature.return_type), expected.and_then(unit_of)) {
                (Some(returned), Some(target)) if *returned == wanted => {
                    filled.units.insert(name.to_string(), target.clone());
                }
                _ => {
                    return Err(Report::refused(
                        span,
                        format!("nothing here says which unit `?{name}` is"),
                        "a call fills a unit parameter from its arguments, or from the type the result goes into (D241, D243)",
                    ));
                }
            }
        }
        let unit = filled.units.get(name).cloned().unwrap_or_default();
        for part in requirement {
            if part == "any.unit" {
                continue;
            }
            let family = unit
                .rsplit_once('.')
                .map_or(unit.as_str(), |(family, _)| family);
            if family != part {
                return Err(Report::refused(
                    span,
                    format!("`{unit}` isn't in the family `{part}`"),
                    "a unit parameter that names a family takes any unit in that family, and only those (D243)",
                ));
            }
        }
        Ok(())
    }

    /// `$a < $b` or `$a + $b` on a `?item`: the operation its requirement lists, done at run
    /// time by whatever type the call filled it with. (D235, D240)
    /// trace: D235, D240
    pub(super) fn generic_operator(
        &mut self,
        word: &str,
        written: &str,
        (left, right): (&Expression, &Expression),
        span: Span,
    ) -> Outcome<(Node, Type)> {
        let left_type = self.type_from_form(left);
        let right_type = self.type_from_form(right);
        let name = [&left_type, &right_type]
            .iter()
            .find_map(|found| match found {
                Some(Type::Parameter(name)) => Some(name.clone()),
                _ => None,
            })
            .unwrap_or_default();
        let Some(parameter) = self.generic.iter().position(|(found, _)| *found == name) else {
            return Err(Report::refused(
                span,
                format!("`?{name}` isn't a type parameter here"),
                WHY_USES,
            ));
        };
        let requirement = self
            .generic
            .get(parameter)
            .map(|(_, requirement)| requirement.clone())
            .unwrap_or_default();
        let own = Type::Parameter(name.clone());
        let operation = requirement
            .iter()
            .flat_map(|part| self.operations_of(part))
            .map(|operation| {
                let own_name = Self::own_name(&operation);
                InterfaceOperatorInfo {
                    operator: operation.operator.clone(),
                    left: put(&operation.left, &own_name, &own),
                    right: put(&operation.right, &own_name, &own),
                    result: put(&operation.result, &own_name, &own),
                }
            })
            .find(|operation| {
                operation.operator == word
                    && left_type
                        .as_ref()
                        .is_none_or(|found| *found == operation.left)
                    && right_type
                        .as_ref()
                        .is_none_or(|found| *found == operation.right)
            });
        let Some(operation) = operation else {
            let follows = if word == written {
                String::new()
            } else {
                format!(", which `{written}` follows from")
            };
            return Err(Report::refused(
                span,
                format!(
                    "`?{name}` needs `{}`, which doesn't list `op {word}`{follows}",
                    requirement.join(" and ")
                ),
                WHY_USES,
            ));
        };
        for (side, wanted) in [(left, &operation.left), (right, &operation.right)] {
            if super::expression::is_number_literal(side) {
                return Err(Report::not_designed(
                    side.span,
                    format!("a bare literal as a `{wanted}`"),
                    "",
                ));
            }
        }
        let (left_node, _) = self.check_expression(left, Some(&operation.left))?;
        let (right_node, _) = self.check_expression(right, Some(&operation.right))?;
        Ok((
            Node {
                span,
                kind: NodeKind::Generic {
                    parameter,
                    operator: word.to_string(),
                    left: Box::new(left_node),
                    right: Box::new(right_node),
                },
            },
            operation.result,
        ))
    }

    /// Whether an expression's type is a type parameter, read from its form.
    pub(super) fn is_generic(&self, expression: &Expression) -> bool {
        matches!(self.type_from_form(expression), Some(Type::Parameter(_)))
    }
}

/// Whether a type has a type or unit parameter inside it, like a list of `f64.?u`.
fn mentions_a_parameter(found: &Type) -> bool {
    match found {
        Type::Parameter(_) => true,
        Type::Integer { unit, .. } | Type::Float { unit, .. } => {
            unit.as_ref().is_some_and(|unit| unit.starts_with('?'))
        }
        Type::List { item, .. } | Type::Array { item, .. } => mentions_a_parameter(item),
        Type::Borrow { inner, .. } => mentions_a_parameter(inner),
        Type::Struct(name) => name.contains('?'),
        _ => false,
    }
}

/// A number type's unit.
fn unit_of(found: &Type) -> Option<&String> {
    match found {
        Type::Integer { unit, .. } | Type::Float { unit, .. } => unit.as_ref(),
        _ => None,
    }
}

/// What a call has filled each type and unit parameter with so far. (D241, D243)
#[derive(Default)]
struct Filled {
    types: HashMap<String, Type>,
    units: HashMap<String, String>,
}

impl Filled {
    /// Matches a parameter's type against an argument's, filling what the parameter leaves
    /// open: `?item` itself, the `?u` in `f64.?u`, and either inside a list or array.
    fn unify(&mut self, pattern: &Type, found: &Type, span: Span) -> Outcome<()> {
        match (pattern, found) {
            (Type::Parameter(name), _) => {
                let found = unbounded(found);
                if let Some(earlier) = self.types.get(name) {
                    if *earlier != found {
                        return Err(Report::refused(
                            span,
                            format!("`?{name}` is `{earlier}` from an earlier argument, and this is `{found}`"),
                            "a call fills each type parameter with one type (D241)",
                        ));
                    }
                }
                self.types.insert(name.clone(), found);
                Ok(())
            }
            (
                Type::Integer {
                    unit: Some(wanted), ..
                }
                | Type::Float {
                    unit: Some(wanted), ..
                },
                Type::Integer { unit, .. } | Type::Float { unit, .. },
            ) if wanted.starts_with('?') => {
                let name = wanted.trim_start_matches('?');
                let Some(unit) = unit else {
                    return Err(Report::refused(
                        span,
                        format!("`{wanted}` takes a unit, and this has none"),
                        "a unit parameter goes where a unit goes, and a call fills it with one (D243)",
                    ));
                };
                if let Some(earlier) = self.units.get(name) {
                    if earlier != unit {
                        return Err(Report::refused(
                            span,
                            format!("`{wanted}` is `{earlier}` from an earlier argument, and this is `{unit}`"),
                            "a call fills each unit parameter with one unit (D241, D243)",
                        ));
                    }
                }
                self.units.insert(name.to_string(), unit.clone());
                Ok(())
            }
            (Type::List { item, .. }, Type::List { item: found, .. })
            | (Type::Array { item, .. }, Type::Array { item: found, .. })
            | (Type::Borrow { inner: item, .. }, Type::Borrow { inner: found, .. }) => {
                self.unify(item, found, span)
            }
            (Type::Borrow { inner, .. }, found) => self.unify(inner, found, span),
            _ if mentions_a_parameter(pattern) => Err(Report::not_designed(
                span,
                format!("working out a type parameter from inside `{pattern}`"),
                "",
            )),
            _ => Ok(()),
        }
    }

    /// A type with what's filled put in.
    fn put_into(&self, found: &Type) -> Type {
        match found {
            Type::Parameter(name) => self
                .types
                .get(name)
                .cloned()
                .unwrap_or_else(|| found.clone()),
            Type::Integer { kind, bound, unit } => Type::Integer {
                kind: *kind,
                bound: *bound,
                unit: self.unit(unit.as_ref()),
            },
            Type::Float {
                kind,
                bound,
                unit,
                rules,
            } => Type::Float {
                kind: *kind,
                bound: *bound,
                unit: self.unit(unit.as_ref()),
                rules: *rules,
            },
            Type::List {
                item,
                min,
                max,
                len_type,
                grow,
            } => Type::List {
                item: Box::new(self.put_into(item)),
                min: *min,
                max: *max,
                len_type: *len_type,
                grow: grow.clone(),
            },
            Type::Array {
                item,
                len,
                len_type,
            } => Type::Array {
                item: Box::new(self.put_into(item)),
                len: *len,
                len_type: *len_type,
            },
            Type::Borrow {
                mutable,
                sources,
                inner,
            } => Type::Borrow {
                mutable: *mutable,
                sources: sources.clone(),
                inner: Box::new(self.put_into(inner)),
            },
            other => other.clone(),
        }
    }

    fn unit(&self, unit: Option<&String>) -> Option<String> {
        let unit = unit?;
        match unit.strip_prefix('?') {
            Some(name) => Some(
                self.units
                    .get(name)
                    .cloned()
                    .unwrap_or_else(|| unit.clone()),
            ),
            None => Some(unit.clone()),
        }
    }
}

/// A type with each type parameter in it filled.
fn fill(found: &Type, filled: &HashMap<String, Type>) -> Type {
    match found {
        Type::Parameter(name) => filled.get(name).cloned().unwrap_or_else(|| found.clone()),
        Type::Borrow {
            mutable,
            sources,
            inner,
        } => Type::Borrow {
            mutable: *mutable,
            sources: sources.clone(),
            inner: Box::new(fill(inner, filled)),
        },
        other => other.clone(),
    }
}

impl Checker<'_> {
    /// `pair<i32, str>`: a generic struct with its type parameters filled, in order, each
    /// meeting its requirement. Its fields are the struct's, with those types put in. (D234)
    /// trace: D234, D235
    pub(super) fn struct_instance(
        &self,
        head: &str,
        written: &[crate::ast::Fill],
        span: Span,
    ) -> Outcome<Type> {
        let Some(info) = self.structs.get(head).cloned() else {
            return Err(Report::refused(
                span,
                format!("there's no struct `{head}`"),
                "(D57)",
            ));
        };
        if written.len() != info.type_parameters.len() {
            let wanted: Vec<String> = info
                .type_parameters
                .iter()
                .map(|(name, _)| format!("?{name}"))
                .collect();
            return Err(Report::refused(
                span,
                format!("`{head}` fills `{}`, in order", wanted.join("`, `")),
                "a generic struct is named with each of its type parameters filled, in `< >` (D234)",
            ));
        }
        let mut filled: HashMap<String, Type> = HashMap::new();
        let mut shown = Vec::new();
        for ((name, requirement), entry) in info.type_parameters.iter().zip(written) {
            if entry.label.is_some() {
                return Err(Report::not_designed(
                    entry.written_type.span,
                    "naming a type parameter where a generic struct is filled",
                    "12.5",
                ));
            }
            let found = self.resolve_type(&entry.written_type)?;
            if self.is_must_give(&found) {
                return Err(Report::not_designed(
                    entry.written_type.span,
                    format!("a `must_Give` type, `{found}`, filling `?{name}`"),
                    "",
                ));
            }
            self.meets(&found, requirement, entry.written_type.span)?;
            shown.push(found.to_string());
            filled.insert(name.clone(), found);
        }
        let instance = format!("{head}<{}>", shown.join(", "));
        if !self.instances.borrow().contains_key(&instance) {
            let fields = info
                .fields
                .iter()
                .map(|(name, found)| (name.clone(), fill(found, &filled)))
                .collect();
            self.instances.borrow_mut().insert(
                instance.clone(),
                super::StructInfo {
                    must_give: info.must_give,
                    packed: info.packed,
                    layout: info.layout,
                    type_parameters: Vec::new(),
                    fields,
                    sources: info.sources.clone(),
                    facts: info.facts.clone(),
                },
            );
        }
        Ok(Type::Struct(instance))
    }
}

impl Checker<'_> {
    /// A call's function, with a generic's type parameters filled from this call, and the
    /// tables the call passes for them. (D241)
    /// trace: D241
    pub(super) fn fill_if_generic(
        &mut self,
        signature: Signature,
        arguments: &[crate::ast::Argument],
        span: Span,
    ) -> Outcome<(Signature, Vec<Table>)> {
        let expected = self.context.call_expected.take();
        if signature.type_parameters.is_empty() {
            return Ok((signature, Vec::new()));
        }
        self.fill_type_parameters(&signature, arguments, expected.as_ref(), span)
    }

    /// A generic struct is built as the instance the line names, as in
    /// `let $p: pair<i32, str> = pair[…];`. (D234, D241)
    /// trace: D234, D241
    pub(super) fn built_instance(
        &self,
        bare: String,
        expected: Option<&Type>,
        span: Span,
    ) -> Outcome<String> {
        let generic = self
            .structs
            .get(&bare)
            .is_some_and(|info| !info.type_parameters.is_empty());
        if !generic {
            return Ok(bare);
        }
        match expected {
            Some(Type::Struct(instance))
                if instance
                    .strip_prefix(bare.as_str())
                    .is_some_and(|rest| rest.starts_with('<')) =>
            {
                Ok(instance.clone())
            }
            _ => Err(Report::not_designed(
                span,
                format!("working out `{bare}`'s type parameters from what builds it"),
                "",
            )),
        }
    }
}

impl Checker<'_> {
    /// The families your own units are in, like `length`, which a unit parameter can need.
    fn unit_families(&self) -> HashSet<String> {
        self.units
            .keys()
            .filter_map(|name| name.split_once('.').map(|(family, _)| family.to_string()))
            .collect()
    }

    /// Whether a type parameter in scope is a unit parameter, or nothing when it isn't one.
    /// (D243)
    pub(super) fn is_unit_parameter(&self, name: &str) -> Option<bool> {
        let (_, requirement) = self.generic.iter().find(|(found, _)| found == name)?;
        let families = self.unit_families();
        Some(
            requirement
                .iter()
                .any(|part| part == "any.unit" || families.contains(part)),
        )
    }
}
