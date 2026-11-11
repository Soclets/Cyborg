//! Structs (D57) and tagged unions (D33): declaring them, building them, and reading their fields.

use std::collections::HashMap;

use crate::checked::{self, ExpressionKind as Kind};
use crate::diagnostic::Diagnostic;
use crate::source::Span;
use crate::syntax::{self, ExpressionKind};

use super::expressions::Typed;
use super::types::Type;
use super::{Body, Check, Checker, Reported};

/// A field of a struct, or of a union's case.
#[derive(Clone, Debug)]
pub struct FieldInfo {
    pub name: String,
    pub value_type: Type,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct StructInfo {
    pub name: String,
    pub fields: Vec<FieldInfo>,
    pub must_give: bool,
}

#[derive(Clone, Debug)]
pub struct CaseInfo {
    pub name: String,
    pub fields: Vec<FieldInfo>,
}

#[derive(Clone, Debug)]
pub struct UnionInfo {
    pub name: String,
    pub cases: Vec<CaseInfo>,
}

/// The program's own types, by name.
#[derive(Default)]
pub struct Aggregates {
    pub structs: Vec<StructInfo>,
    pub unions: Vec<UnionInfo>,
    pub names: HashMap<String, Type>,
}

impl Checker<'_> {
    /// Declares every struct and union by name, then reads their fields, so they can refer to each other.
    pub fn declare_aggregates(&mut self, file: &syntax::File) {
        for item in &file.items {
            match &item.kind {
                syntax::ItemKind::Struct(declared) => {
                    let name = declared.name.text();
                    let id = self.aggregates.structs.len();
                    let must_give = declared.keyword.text() == "struct.must_Give";
                    self.aggregates.structs.push(StructInfo { name: name.clone(), fields: Vec::new(), must_give });
                    self.aggregates.names.insert(name.clone(), Type::Struct { id, name });
                }
                syntax::ItemKind::Union(declared) => {
                    let name = declared.name.text();
                    let keyword = declared.keyword.text();
                    if keyword == "union" {
                        self.error(Diagnostic::refused_fix(
                            declared.keyword.span,
                            "union.tagged",
                            "\"union\" means opposite things to a C programmer and a Rust one, so it says which (D33)",
                        ));
                        continue;
                    }
                    let id = self.aggregates.unions.len();
                    self.aggregates.unions.push(UnionInfo { name: name.clone(), cases: Vec::new() });
                    self.aggregates.names.insert(name.clone(), Type::Union { id, name });
                }
                _ => {}
            }
        }
        let mut struct_id = 0;
        let mut union_id = 0;
        for item in &file.items {
            match &item.kind {
                syntax::ItemKind::Struct(declared) => {
                    let name = declared.name.text();
                    let expected = format!("end {} {name}", declared.keyword.text());
                    self.check_closer(&declared.end, &expected, super::CLOSER_WHY);
                    let fields = self.fields(
                        &declared.fields,
                        &name,
                        "fields",
                        "a field read is a plain `.`, so a field's name says which (D57)",
                    );
                    if let Some(info) = self.aggregates.structs.get_mut(struct_id) {
                        info.fields = fields;
                    }
                    struct_id += 1;
                }
                syntax::ItemKind::Union(declared) if declared.keyword.text() != "union" => {
                    let name = declared.name.text();
                    let expected = format!("end {} {name}", declared.keyword.text());
                    self.check_closer(&declared.end, &expected, super::CLOSER_WHY);
                    let mut cases: Vec<CaseInfo> = Vec::new();
                    for case in &declared.cases {
                        if cases.iter().any(|earlier| earlier.name == case.name.text) {
                            self.error(Diagnostic::refused(
                                case.name.span,
                                format!("`{name}` has two cases called `{}`", case.name.text),
                                "a union's cases are narrowings, so a case's name says which (D33)",
                            ));
                            continue;
                        }
                        let fields = self.fields(
                            &case.fields,
                            &name,
                            "fields",
                            "a field read is a plain `.`, so a field's name says which (D57)",
                        );
                        cases.push(CaseInfo { name: case.name.text.clone(), fields });
                    }
                    if let Some(info) = self.aggregates.unions.get_mut(union_id) {
                        info.cases = cases;
                    }
                    union_id += 1;
                }
                _ => {}
            }
        }
        self.program.structs = self
            .aggregates
            .structs
            .iter()
            .map(|info| info.fields.iter().map(|field| field.value_type.machine()).collect())
            .collect();
        self.program.unions = self
            .aggregates
            .unions
            .iter()
            .map(|info| {
                info.cases
                    .iter()
                    .map(|case| case.fields.iter().map(|field| field.value_type.machine()).collect())
                    .collect()
            })
            .collect();
    }

    /// Reads a list of fields, refusing one named twice.
    fn fields(&mut self, written: &[syntax::Field], owner: &str, _kind: &str, why: &str) -> Vec<FieldInfo> {
        let mut fields: Vec<FieldInfo> = Vec::new();
        for field in written {
            if fields.iter().any(|earlier| earlier.name == field.name.text) {
                self.error(Diagnostic::refused(
                    field.name.span,
                    format!("`{owner}` has two fields called `{}`", field.name.text),
                    why,
                ));
                continue;
            }
            let value_type = self.resolve_type(&field.written_type).unwrap_or(Type::Unknown);
            fields.push(FieldInfo { name: field.name.text.clone(), value_type, span: field.span });
        }
        fields
    }

    /// A union's case written as a path, `status.ok`, if the path names one.
    pub fn union_case(&self, path: &syntax::Path) -> Option<(usize, String, String)> {
        let text = self.resolve_path(path);
        let (union_name, case_name) = text.rsplit_once('.')?;
        // An alias names the union's cases too, which are still the union's own (D134).
        let Some(Type::Union { id, name }) = self.aggregates.names.get(union_name) else { return None };
        Some((*id, name.clone(), case_name.to_string()))
    }

    /// A case as written, with an alias of the union read as the union (D134).
    pub fn union_case_written(&self, path: &syntax::Path) -> String {
        match self.union_case(path) {
            Some((_, union_name, case_name)) => format!("{union_name}.{case_name}"),
            None => self.resolve_path(path),
        }
    }

    /// `status.ok`: a case with no values.
    pub fn case_value(&mut self, expression: &syntax::Expression, path: &syntax::Path) -> Check<Option<Typed>> {
        let Some((id, union_name, case_name)) = self.union_case(path) else { return Ok(None) };
        let union = self.aggregates.unions[id].clone();
        let Some(case) = union.cases.iter().position(|case| case.name == case_name) else {
            return Err(self.error(Diagnostic::refused(
                expression.span,
                format!("`{union_name}` has no case `{case_name}`"),
                "a union's cases are narrowings (D33)",
            )));
        };
        if !union.cases[case].fields.is_empty() {
            return Err(self.error(Diagnostic::refused(
                expression.span,
                format!("`{union_name}.{case_name}` has values, which go in `[ ]`"),
                "a case's values go in `[ ]` (D33)",
            )));
        }
        let value_type = Type::Union { id, name: union_name };
        Ok(Some(Typed::new(value_type, Kind::BuildCase { case, fields: Vec::new() })))
    }

    /// `reading[celsius = 21]` or `status.failed[code = 3]`: building a struct or a case, with `=` setting each field
    /// (D57, D33).
    pub fn build(
        &mut self,
        body: &mut Body,
        expression: &syntax::Expression,
        path: &syntax::Path,
        labels: &[syntax::Label],
    ) -> Check<Option<Typed>> {
        let name = self.resolve_path(path);
        if self.type_names.declared.get(&name).is_some_and(|named| named.new) {
            return Err(self.error(Diagnostic::not_designed(
                expression.span,
                "building a `type.new` by its own name, rather than crossing with `as`",
                "not decided yet (standard 12.1)",
            )));
        }
        let (fields, value_type, owner, case) = match self.aggregates.names.get(&name).cloned() {
            Some(Type::Struct { id, name }) => {
                (self.aggregates.structs[id].fields.clone(), Type::Struct { id, name: name.clone() }, name, None)
            }
            _ => {
                let Some((id, union_name, case_name)) = self.union_case(path) else { return Ok(None) };
                let union = self.aggregates.unions[id].clone();
                let Some(case) = union.cases.iter().position(|case| case.name == case_name) else {
                    return Err(self.error(Diagnostic::refused(
                        path.span,
                        format!("`{union_name}` has no case `{case_name}`"),
                        "a union's cases are narrowings (D33)",
                    )));
                };
                let owner = format!("{union_name}.{case_name}");
                (union.cases[case].fields.clone(), Type::Union { id, name: union_name }, owner, Some(case))
            }
        };
        let mut values: Vec<Option<checked::Expression>> = vec![None; fields.len()];
        let mut failed = false;
        for label in labels {
            let label_name = label.name.text();
            if !label.sets {
                let colon_end = label.value.span.start as usize;
                let span = Span::new(self.file, label.name.span.start as usize, colon_end.saturating_sub(1));
                self.error(Diagnostic::refused_fix(
                    label.name.span.to(span),
                    format!("{label_name} ="),
                    "building one uses `[ ]`, with `=` setting a field (D57)",
                ));
                failed = true;
                continue;
            }
            let Some(index) = fields.iter().position(|field| field.name == label_name) else {
                self.error(Diagnostic::refused(
                    label.name.span,
                    format!("`{owner}` has no field `{label_name}`"),
                    "building one uses `[ ]`, with `=` setting each field (D57)",
                ));
                failed = true;
                continue;
            };
            match self.value_into(body, &label.value, &fields[index].value_type) {
                Ok(value) => values[index] = Some(value),
                Err(Reported) => failed = true,
            }
        }
        if failed {
            return Err(Reported);
        }
        if let Some(missing) = values.iter().position(Option::is_none) {
            return Err(self.error(Diagnostic::refused(
                expression.span,
                format!("building `{owner}` needs `{}`", fields[missing].name),
                "building one uses `[ ]`, with `=` setting each field (D57)",
            )));
        }
        let values: Vec<checked::Expression> = values.into_iter().flatten().collect();
        let kind = match case {
            Some(case) => Kind::BuildCase { case, fields: values },
            None => Kind::Build(values),
        };
        Ok(Some(Typed::new(value_type, kind)))
    }

    /// A field read, `$r.celsius` (D57). A union's field is read only where its case is known (D9).
    pub fn field(&mut self, body: &mut Body, base: &syntax::Expression, field: &syntax::Name) -> Check<Typed> {
        // A failure named in an arm is read through its fields (D139).
        if let ExpressionKind::Variable(name) = &base.kind
            && let Some(variable) = body.lookup(&name.text).cloned()
            && let Type::Failure { name: failure, fields, .. } = &variable.value_type
        {
            let Some(index) = fields.iter().position(|(each, _)| *each == field.text) else {
                return Err(self.error(Diagnostic::refused(
                    field.span,
                    format!("`{failure}` carries no `{}`", field.text),
                    "a failure can carry data in `[ ]`, and `case negative $f:` reads it (D47, D139)",
                )));
            };
            let case = variable.known_case.unwrap_or(0);
            let value =
                checked::Expression { kind: Kind::Local(variable.local), machine_type: variable.value_type.machine() };
            let mut typed = Typed::new(fields[index].1.clone(), Kind::CaseField { base: Box::new(value), case, index });
            typed.variable = Some(format!("{}.{}", name.text, field.text));
            return Ok(typed);
        }
        let typed = self.expression(body, base, None)?;
        match &typed.value_type {
            Type::New { .. } => Err(self.error(Diagnostic::not_designed(
                field.span,
                "a field of a `type.new`, rather than crossing with `as` first",
                "not decided yet (standard 12.1)",
            ))),
            Type::Struct { id, name } => {
                let info = self.aggregates.structs[*id].clone();
                let Some(index) = info.fields.iter().position(|each| each.name == field.text) else {
                    return Err(self.error(Diagnostic::refused(
                        field.span,
                        format!("`{name}` has no field `{}`", field.text),
                        "a field read is a plain `.` (D57)",
                    )));
                };
                let value_type = info.fields[index].value_type.clone();
                let mut result = Typed::new(value_type, Kind::Field { base: Box::new(typed.value), index });
                result.variable = typed.variable.map(|variable| format!("{variable}.{}", field.text));
                Ok(result)
            }
            Type::Union { id, name } => {
                let info = self.aggregates.unions[*id].clone();
                let known = match &base.kind {
                    ExpressionKind::Variable(variable) => body.lookup(&variable.text).and_then(|each| each.known_case),
                    _ => None,
                };
                let holding = info.cases.iter().position(|case| case.fields.iter().any(|each| each.name == field.text));
                match (known, holding) {
                    (Some(case), _) if !info.cases[case].fields.iter().any(|each| each.name == field.text) => Err(self
                        .error(Diagnostic::refused(
                            field.span,
                            format!("`{name}.{}` has no field `{}`", info.cases[case].name, field.text),
                            "a case's values go in `[ ]` (D33)",
                        ))),
                    (Some(case), _) if info.cases[case].fields.iter().any(|each| each.name == field.text) => {
                        let index =
                            info.cases[case].fields.iter().position(|each| each.name == field.text).unwrap_or(0);
                        let value_type = info.cases[case].fields[index].value_type.clone();
                        Ok(Typed::new(value_type, Kind::CaseField { base: Box::new(typed.value), case, index }))
                    }
                    (_, Some(_)) if matches!(&base.kind, ExpressionKind::Variable(variable) if body.matching.contains(&variable.text)) =>
                    {
                        let shown = self.slice(base.span.to(field.span)).to_string();
                        Err(self.error(Diagnostic::not_designed(
                            base.span.to(field.span),
                            format!("reading `{shown}` in a `match` arm, rather than through a name for the case"),
                            super::NOT_DECIDED,
                        )))
                    }
                    (_, Some(case)) => {
                        let shown = self.slice(base.span).to_string();
                        Err(self.error(Diagnostic::refused(
                            base.span.to(field.span),
                            format!("`{shown}` isn't known to be `{name}.{}` here", info.cases[case].name),
                            format!(
                                "inside `if {shown}: {name}.{}:` the compiler trusts it, and outside that's a compile error (D9)",
                                info.cases[case].name
                            ),
                        )))
                    }
                    _ => Err(self.error(Diagnostic::refused(
                        field.span,
                        format!("`{name}` has no field `{}`", field.text),
                        "a field read is a plain `.` (D57)",
                    ))),
                }
            }
            Type::Unknown => Err(Reported),
            other => Err(self.not_yet(field.span, format!("a field of `{other}`"))),
        }
    }
}
