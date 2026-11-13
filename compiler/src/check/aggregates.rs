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
    /// The sources its borrows name, in order (D171).
    pub sources: Vec<String>,
    /// Each `lasts_at_least` fact between them: the longer one, then the other.
    pub facts: Vec<(String, String)>,
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
    /// The sources its cases' borrows name, in order (D176).
    pub sources: Vec<String>,
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
                syntax::ItemKind::Struct(declared) if has_type_parameters(&declared.generics) => {
                    // A generic struct is a pattern, with a struct made for each set of types that fills it (D234).
                    let Ok(parameters) = self.type_parameters(Some(&declared.name), &declared.generics) else {
                        continue;
                    };
                    self.generics.structs.insert(
                        declared.name.text(),
                        super::generics::GenericStruct { syntax: declared.clone(), parameters },
                    );
                }
                syntax::ItemKind::Struct(declared) => {
                    let name = declared.name.text();
                    let id = self.aggregates.structs.len();
                    let must_give = declared.keyword.text() == "struct.must_Give";
                    let (sources, facts) = self.declare_sources(&declared.generics);
                    let sources = sources.into_iter().map(|(source, _)| source).collect();
                    self.aggregates.structs.push(StructInfo {
                        name: name.clone(),
                        fields: Vec::new(),
                        must_give,
                        sources,
                        facts,
                    });
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
                    let (sources, _) = self.declare_sources(&declared.generics);
                    let sources = sources.into_iter().map(|(source, _)| source).collect();
                    self.aggregates.unions.push(UnionInfo { name: name.clone(), cases: Vec::new(), sources });
                    self.aggregates.names.insert(name.clone(), Type::Union { id, name });
                }
                _ => {}
            }
        }
        let mut struct_id = 0;
        let mut union_id = 0;
        for item in &file.items {
            match &item.kind {
                syntax::ItemKind::Struct(declared) if has_type_parameters(&declared.generics) => {
                    let name = declared.name.text();
                    let expected = format!("end {} {name}", declared.keyword.text());
                    self.check_closer(&declared.end, &expected, super::CLOSER_WHY);
                }
                syntax::ItemKind::Struct(declared) => {
                    let name = declared.name.text();
                    let expected = format!("end {} {name}", declared.keyword.text());
                    self.check_closer(&declared.end, &expected, super::CLOSER_WHY);
                    let declared_sources = source_spans(&declared.generics);
                    self.source_scope = Some(super::sources::SourceScope {
                        names: declared_sources.iter().map(|(source, _)| source.clone()).collect(),
                        aggregate: true,
                    });
                    let fields = self.fields(
                        &declared.fields,
                        &name,
                        "fields",
                        "a field read is a plain `.`, so a field's name says which (D57)",
                    );
                    self.source_scope = None;
                    let types: Vec<&Type> = fields.iter().map(|field| &field.value_type).collect();
                    self.check_sources_used(&declared_sources, &types);
                    // A struct holding a `must_Give` value says so itself (D177).
                    let says_it = declared.keyword.text() == "struct.must_Give";
                    if !says_it
                        && let Some(field) = declared
                            .fields
                            .iter()
                            .zip(&fields)
                            .find(|(_, info)| self.must_give(&info.value_type))
                            .map(|(field, _)| field)
                    {
                        self.error(Diagnostic::refused_fix(
                            field.written_type.span,
                            format!("struct.must_Give {name}"),
                            "a struct holding a `must_Give` value is `must_Give` itself, and nothing is implicit (D177, D3)",
                        ));
                    }
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
                    let declared_sources = source_spans(&declared.generics);
                    self.source_scope = Some(super::sources::SourceScope {
                        names: declared_sources.iter().map(|(source, _)| source.clone()).collect(),
                        aggregate: true,
                    });
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
                        if let Some(field) =
                            case.fields.iter().zip(&fields).find(|(_, info)| self.must_give(&info.value_type))
                        {
                            self.error(Diagnostic::not_designed(
                                field.0.written_type.span,
                                "a `must_Give` value in a `union.tagged` case",
                                "not decided yet (standard 12.8)",
                            ));
                        }
                        cases.push(CaseInfo { name: case.name.text.clone(), fields });
                    }
                    self.source_scope = None;
                    let types: Vec<&Type> =
                        cases.iter().flat_map(|case| case.fields.iter().map(|field| &field.value_type)).collect();
                    self.check_sources_used(&declared_sources, &types);
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
        expected: Option<&Type>,
    ) -> Check<Option<Typed>> {
        let mut name = self.resolve_path(path);
        // A generic struct is built as the copy the type it goes into names, as in `let $p: pair<i32, str> = pair[…]`.
        if self.generics.structs.contains_key(&name) {
            match expected {
                Some(Type::Struct { name: filled, .. }) if filled.starts_with(&format!("{name}<")) => {
                    name.clone_from(filled);
                }
                _ => {
                    return Err(self.not_yet(
                        expression.span,
                        format!("building a `{name}` where nothing says what fills its type parameters"),
                    ));
                }
            }
        }
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
            let value = if let Type::Borrow { .. } = fields[index].value_type {
                self.built_borrow(body, &owner, expression.span, &label.value, &fields[index].value_type)
            } else {
                self.value_into(body, &label.value, &fields[index].value_type)
            };
            match value {
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
        if field.text == "len"
            && let Some(length) = self.length(&typed)
        {
            return Ok(length);
        }
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
                // A field that borrows reads as what it borrows (D171).
                let value_type = info.fields[index].value_type.read_as().clone();
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
                        let value_type = info.cases[case].fields[index].value_type.read_as().clone();
                        // Reading through a test's trust, which a `change` or `Lend.mut` ends (D104).
                        if let ExpressionKind::Variable(variable) = &base.kind
                            && let Some(found) = body.lookup(&variable.text)
                        {
                            let place = base.span.to(field.span);
                            body.events.push(crate::flow::Event::TrustedRead {
                                local: found.local,
                                spot: crate::flow::Spot { variable: base.span, place, site: place, marker: place },
                                what: format!("{name}.{}", info.cases[case].name),
                            });
                        }
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

/// A declaration's sources with where each is written, for errors about them.
fn source_spans(generics: &[syntax::Generic]) -> Vec<(String, Span)> {
    let mut sources: Vec<(String, Span)> = Vec::new();
    for generic in generics {
        if generic.kind == syntax::GenericKind::Source && !sources.iter().any(|(name, _)| *name == generic.name.text) {
            sources.push((generic.name.text.clone(), generic.name.span));
        }
    }
    sources
}

fn has_type_parameters(generics: &[syntax::Generic]) -> bool {
    generics.iter().any(|generic| generic.kind == syntax::GenericKind::Type)
}

impl Checker<'_> {
    /// `pair<i32, str>`: the struct made for a generic struct with these types filling its type parameters, made the
    /// first time it's named (D234, D239).
    pub fn generic_struct(&mut self, written: &syntax::Type, name: &str, fill: &[syntax::TypeArgument]) -> Check<Type> {
        let Some(generic) = self.generics.structs.get(name).cloned() else { return Err(Reported) };
        let mut filled = Vec::new();
        for argument in fill {
            match &argument.value {
                syntax::TypeArgumentValue::Type(each) if argument.label.is_none() => {
                    filled.push(self.resolve_type(each)?)
                }
                _ => return Err(self.not_yet(argument.span, "this in a generic struct's `< >`".to_string())),
            }
        }
        if filled.len() != generic.parameters.len() {
            let wanted: Vec<String> = generic.parameters.iter().map(|each| format!("`?{}`", each.name)).collect();
            return Err(self.error(Diagnostic::refused(
                written.span,
                format!("`{name}` is filled with a type for each of {}", wanted.join(", ")),
                "a generic struct says what fills each of its type parameters, in order (D234)",
            )));
        }
        for (parameter, value_type) in generic.parameters.iter().zip(&filled) {
            self.meets_struct(written.span, &parameter.needs, value_type)?;
        }
        let shown = format!("{name}<{}>", filled.iter().map(ToString::to_string).collect::<Vec<_>>().join(", "));
        if let Some(found) = self.aggregates.names.get(&shown) {
            return Ok(found.clone());
        }
        // Its number is the next struct's, after any result layouts the program's functions made.
        let id = self.program.structs.len();
        while self.aggregates.structs.len() < id {
            self.aggregates.structs.push(StructInfo {
                name: String::new(),
                fields: Vec::new(),
                must_give: false,
                sources: Vec::new(),
                facts: Vec::new(),
            });
        }
        self.aggregates.structs.push(StructInfo {
            name: shown.clone(),
            fields: Vec::new(),
            must_give: generic.syntax.keyword.text() == "struct.must_Give",
            sources: Vec::new(),
            facts: Vec::new(),
        });
        self.program.structs.push(Vec::new());
        let made = Type::Struct { id, name: shown.clone() };
        self.aggregates.names.insert(shown.clone(), made.clone());
        let outer = self.type_scope.len();
        self.type_scope.extend(generic.parameters.iter().zip(&filled).map(|(parameter, value_type)| {
            (parameter.name.clone(), super::generics::Bound::Filled(value_type.clone()))
        }));
        let fields = self.fields(
            &generic.syntax.fields,
            &shown,
            "fields",
            "a field read is a plain `.`, so a field's name says which (D57)",
        );
        self.type_scope.truncate(outer);
        if let Some(slot) = self.program.structs.get_mut(id) {
            *slot = fields.iter().map(|field| field.value_type.machine()).collect();
        }
        if let Some(info) = self.aggregates.structs.get_mut(id) {
            info.fields = fields;
        }
        Ok(made)
    }
}
