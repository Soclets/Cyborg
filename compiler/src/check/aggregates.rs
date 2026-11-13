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
    /// How its fields are laid out in memory (D319, D388).
    pub layout: Layout,
    /// The sources its borrows name, in order (D171).
    pub sources: Vec<String>,
    /// Each `lasts_at_least` fact between them: the longer one, then the other.
    pub facts: Vec<(String, String)>,
}

/// A struct's layout (D319, D388): the written order with natural padding, sorted by alignment with `.compact`, the
/// compiler's own choice with `.any_order`, or the written order with no padding with `.packed`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Layout {
    Written,
    Compact,
    AnyOrder,
    Packed,
    /// C's union, whose fields share the same bytes (D35, D398).
    Untagged,
}

impl Layout {
    #[must_use]
    pub fn of_keyword(keyword: &str) -> Layout {
        if keyword.contains("untagged") {
            Layout::Untagged
        } else if keyword.contains(".compact") {
            Layout::Compact
        } else if keyword.contains(".any_order") {
            Layout::AnyOrder
        } else if keyword.contains(".packed") {
            Layout::Packed
        } else {
            Layout::Written
        }
    }
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
                    self.untagged_keyword(declared);
                    let id = self.aggregates.structs.len();
                    let must_give = declared.keyword.text() == "struct.must_Give";
                    let (sources, facts) = self.declare_sources(&declared.generics);
                    let sources = sources.into_iter().map(|(source, _)| source).collect();
                    self.aggregates.structs.push(StructInfo {
                        name: name.clone(),
                        fields: Vec::new(),
                        must_give,
                        layout: Layout::of_keyword(&declared.keyword.text()),
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
                    // A struct with two narrowings was already said to be undecided, closer and all.
                    if declared.keyword.segments.len() <= 2 {
                        self.check_closer(&declared.end, &expected, super::CLOSER_WHY);
                    }
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
                    // `.compact` lays fields out largest alignment first, ties in the written order, and `.any_order`
                    // is this compiler's choice, the same (D388). Fields are found by name, so the order is theirs.
                    let mut fields = fields;
                    let layout = Layout::of_keyword(&declared.keyword.text());
                    if matches!(layout, Layout::Compact | Layout::AnyOrder) {
                        let mut aligned: Vec<(u64, FieldInfo)> = fields
                            .into_iter()
                            .map(|field| {
                                (self.footprint(&field.value_type).map_or(8, |footprint| footprint.align), field)
                            })
                            .collect();
                        aligned.sort_by(|a, b| b.0.cmp(&a.0));
                        fields = aligned.into_iter().map(|(_, field)| field).collect();
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
        let mut machine_structs = Vec::new();
        for info in &self.aggregates.structs {
            // C's union holds all its bytes as one number, read back as each field's type (D398).
            if info.layout == Layout::Untagged {
                let bytes = self.footprint(&Type::Struct { id: machine_structs.len(), name: info.name.clone() });
                let bits = bytes.map_or(64, |footprint| u32::try_from(footprint.size * 8).unwrap_or(64));
                machine_structs.push(vec![checked::MachineType::Integer { bits, signed: false }]);
                continue;
            }
            machine_structs.push(info.fields.iter().map(|field| field.value_type.machine()).collect());
        }
        self.program.structs = machine_structs;
        self.program.packed = self.aggregates.structs.iter().map(|info| info.layout == Layout::Packed).collect();
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
        if let Some(Type::Struct { id, name }) = self.aggregates.names.get(&name).cloned()
            && self.aggregates.structs[id].layout == Layout::Untagged
        {
            return self.untagged_build(body, expression, id, &name, labels).map(Some);
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
    /// C's union's keyword: `union.@csup::untagged`, only inside a shield, and its older spellings refused (D35, D38,
    /// D399).
    fn untagged_keyword(&mut self, declared: &syntax::Struct) {
        let keyword = declared.keyword.text();
        match keyword.as_str() {
            "union.untagged" => {
                self.error(Diagnostic::refused_fix(
                    declared.keyword.span,
                    "union.@csup::untagged",
                    "C support is the limb `@csup`, and that union must be inside a shield (D35, D38, D399)",
                ));
            }
            "@csup::union.untagged" => {
                self.error(Diagnostic::refused_fix(
                    declared.keyword.span,
                    "union.@csup::untagged",
                    "the limb's untagged union joins the base `union` family, as `inline.@llvm::ir` joins `inline.` (D399, D227)",
                ));
            }
            "union.@csup::untagged" if !self.shielded.contains(&declared.name.text()) => {
                self.error(Diagnostic::refused(
                    declared.keyword.span,
                    "`union.@csup::untagged` must be inside a shield",
                    "a shield is the only place dangerous code may live (D34, D38)",
                ));
            }
            _ => {}
        }
    }

    /// `number[f = 1.5, rest = all 0]`: C's union built by naming the one field written, with the rest of its bytes
    /// zeroed, in a shield (D398, D401, D402).
    fn untagged_build(
        &mut self,
        body: &mut Body,
        expression: &syntax::Expression,
        id: usize,
        name: &str,
        labels: &[syntax::Label],
    ) -> Check<Typed> {
        if self.shields == 0 {
            return Err(self.error(Diagnostic::refused(
                expression.span,
                format!("building a `union.@csup::untagged {name}` outside a shield"),
                "C's union is declared, built and read only in shields, where the reason vouches for it (D38, D398)",
            )));
        }
        let written: Vec<&syntax::Label> = labels.iter().filter(|label| label.name.text() != "rest").collect();
        if written.len() != 1 {
            return Err(self.error(Diagnostic::refused(
                expression.span,
                format!(
                    "this builds `{name}` with {} fields, and C's union is built by naming exactly the one written",
                    written.len()
                ),
                "a C union's fields share the same bytes, so building one names the field it writes (D398)",
            )));
        }
        match labels.iter().find(|label| label.name.text() == "rest") {
            None => {
                let end =
                    Span { file: expression.span.file, start: expression.span.end - 1, end: expression.span.end - 1 };
                return Err(self.error(
                    Diagnostic::refused_fix(
                        expression.span,
                        "rest = all 0",
                        "the bytes a union's written field doesn't cover are a choice, so the build says it, and `rest = all 0` sets them to 0 (D401, D402)",
                    )
                    .with_edits(vec![crate::diagnostic::Edit { span: end, replacement: ", rest = all 0".to_string() }]),
                ));
            }
            Some(rest) if matches!(rest.value.kind, ExpressionKind::Empty) => {
                return Err(self.error(Diagnostic::refused_fix(
                    rest.value.span,
                    "all 0",
                    "a union fills the rest of its bytes with `rest = all 0`, as `all 0` fills an array (D402, D103)",
                )));
            }
            Some(_) => {}
        }
        let label = written[0];
        let field_name = label.name.text();
        let info = self.aggregates.structs[id].clone();
        let Some(field) = info.fields.iter().find(|each| each.name == field_name) else {
            return Err(self.error(Diagnostic::refused(
                label.name.span,
                format!("`{name}` has no field `{field_name}`"),
                "a field read is a plain `.` (D57)",
            )));
        };
        let value = self.value_into(body, &label.value, &field.value_type)?;
        let bytes = self.program.structs.get(id).and_then(|fields| fields.first()).cloned();
        let Some(bytes @ checked::MachineType::Integer { .. }) = bytes else {
            return Err(self.not_yet(expression.span, format!("building `{name}`")));
        };
        let held = checked::Expression { kind: Kind::UnionBits(Box::new(value)), machine_type: bytes };
        Ok(Typed::new(Type::Struct { id, name: name.to_string() }, Kind::Build(vec![held])))
    }

    /// `$n.i`: C's union read as one of its fields, the same bytes taken as that type, and checked like any value of
    /// it (D398, D401).
    fn untagged_read(&mut self, id: usize, typed: Typed, field: &syntax::Name, span: Span) -> Check<Typed> {
        use super::expressions::failure;
        use crate::diagnostic::Kind as DiagnosticKind;
        let info = self.aggregates.structs[id].clone();
        let Some(found) = info.fields.iter().find(|each| each.name == field.text) else {
            return Err(self.error(Diagnostic::refused(
                field.span,
                format!("`{}` has no field `{}`", info.name, field.text),
                "a field read is a plain `.` (D57)",
            )));
        };
        let field_type = found.value_type.clone();
        let bytes = self
            .program
            .structs
            .get(id)
            .and_then(|fields| fields.first())
            .cloned()
            .unwrap_or(checked::MachineType::I64);
        let held =
            checked::Expression { kind: Kind::Field { base: Box::new(typed.value), index: 0 }, machine_type: bytes };
        // trace: D401
        let read = match &field_type {
            Type::Bool => {
                let byte =
                    checked::Expression { kind: Kind::Bits(Box::new(held)), machine_type: checked::MachineType::U8 };
                let check = failure(
                    self,
                    Diagnostic::new(
                        DiagnosticKind::CheckFailed,
                        span,
                        "this reads a `bool` from the byte \u{1}, which is neither 0 nor 1",
                        "a union's field is checked like any value of its type, and a `bool` is 0 or 1 (D401)",
                    ),
                    &[],
                );
                let fitted = checked::Expression {
                    kind: Kind::Fit { value: Box::new(byte), min: 0, max: 1, failure: Box::new(check) },
                    machine_type: checked::MachineType::U8,
                };
                checked::Expression { kind: Kind::Bits(Box::new(fitted)), machine_type: checked::MachineType::Bool }
            }
            Type::Float { bits, kind, .. } if *kind != super::types::FloatKind::Ieee => {
                let raw = checked::Expression { kind: Kind::Bits(Box::new(held)), machine_type: field_type.machine() };
                let check = failure(
                    self,
                    Diagnostic::new(
                        DiagnosticKind::CheckFailed,
                        span,
                        format!("this reads a `{field_type}` from bits that make NaN or infinity"),
                        "a union's field is checked like any value of its type, and a float can't be NaN or infinity (D401, D158)",
                    ),
                    &[],
                );
                let most = if *bits == 32 { f64::from(f32::MAX) } else { f64::MAX };
                checked::Expression {
                    kind: Kind::FitFloat { value: Box::new(raw), min: -most, max: most, failure: Box::new(check) },
                    machine_type: field_type.machine(),
                }
            }
            Type::Integer { .. } | Type::Float { .. } => {
                checked::Expression { kind: Kind::Bits(Box::new(held)), machine_type: field_type.machine() }
            }
            _ => return Err(self.not_yet(span, format!("reading a `{field_type}` from C's union"))),
        };
        Ok(Typed::new(field_type, read.kind))
    }

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
            Type::Struct { id, .. } if self.aggregates.structs[*id].layout == Layout::Untagged => {
                self.untagged_read(*id, typed, field, base.span.to(field.span))
            }
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
                layout: Layout::Written,
                sources: Vec::new(),
                facts: Vec::new(),
            });
        }
        self.aggregates.structs.push(StructInfo {
            name: shown.clone(),
            fields: Vec::new(),
            must_give: generic.syntax.keyword.text() == "struct.must_Give",
            layout: Layout::of_keyword(&generic.syntax.keyword.text()),
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
