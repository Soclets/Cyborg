//! Borrows inside structs and union cases (D171, D176): a type names its sources in backticks, a field borrows from
//! one, and a type that uses it fills each source, in order. A variable of such a type is a stored borrow of what fills
//! its sources, so the flow pass follows it like any other (D169).

use crate::diagnostic::Diagnostic;
use crate::source::Span;
use crate::syntax;

use super::types::Type;
use super::{Body, Check, Checker, NOT_DECIDED, Reported};

const ORDER_WHY: &str = "a type fills every source, in the declared order, and leaving one out is refused (D171)";
const UNKNOWN_WHY: &str = "a source is declared in backticks on the struct, union or function it belongs to (D171)";

/// The sources in scope while a type is read: a struct's or union's while its fields are, or a function's while its
/// parameters and result are.
#[derive(Clone, Debug, Default)]
pub struct SourceScope {
    pub names: Vec<String>,
    /// Inside a struct or union, where a borrow names a source in backticks.
    pub aggregate: bool,
}

/// A source as a type keeps it: `` `text` ``, with its backticks, apart from a variable like `$r`.
#[must_use]
pub fn quoted(name: &str) -> String {
    format!("`{name}`")
}

impl Checker<'_> {
    /// A struct's or union's sources, `` <`text`, `table`: lasts_at_least `text`> ``, with each fact that one lasts at
    /// least as long as another.
    pub fn declare_sources(&mut self, generics: &[syntax::Generic]) -> (Vec<(String, Span)>, Vec<(String, String)>) {
        let mut sources: Vec<(String, Span)> = Vec::new();
        let mut facts = Vec::new();
        for generic in generics {
            if generic.kind != syntax::GenericKind::Source {
                continue;
            }
            if sources.iter().any(|(name, _)| *name == generic.name.text) {
                self.error(Diagnostic::refused(
                    generic.name.span,
                    format!("`{}` is named twice", generic.name.text),
                    "each source has its own name (D171)",
                ));
                continue;
            }
            if let Some(syntax::Requirement::LastsAtLeast(other)) = &generic.requirement {
                facts.push((generic.name.text.clone(), other.text.clone()));
            }
            sources.push((generic.name.text.clone(), generic.name.span));
        }
        (sources, facts)
    }

    /// Each source is used by a field, or it says nothing (D171).
    pub fn check_sources_used(&mut self, sources: &[(String, Span)], types: &[&Type]) {
        // A field whose type was refused may have been the one using it.
        if types.iter().any(|value_type| value_type.is_unknown()) {
            return;
        }
        for (name, span) in sources {
            let used = types.iter().any(|value_type| match value_type {
                Type::Borrow { sources, .. } => sources.contains(&quoted(name)),
                _ => false,
            });
            if !used {
                self.error(Diagnostic::refused(
                    *span,
                    format!("`{name}` isn't used by any field"),
                    "each source is used by a field, or it says nothing (D171)",
                ));
            }
        }
    }

    /// One source of a borrow type: a variable, `program`, or a source in backticks, which has to be in scope.
    pub fn borrow_source(&mut self, type_span: Span, source: &syntax::Name, mutable: bool) -> Check<String> {
        let backticked = self.slice(source.span).starts_with('`');
        let scope = self.source_scope.clone();
        if scope.as_ref().is_some_and(|scope| scope.aggregate) {
            if mutable {
                return Err(self.error(Diagnostic::not_designed(
                    type_span,
                    "a `Lend.mut` borrow inside a struct or union",
                    NOT_DECIDED,
                )));
            }
            if source.text == "program" {
                return Err(self.error(Diagnostic::not_designed(
                    type_span,
                    "a field borrowing `from program`",
                    "not decided yet (standard 12.8)",
                )));
            }
            if !backticked {
                return Err(self.error(Diagnostic::refused(
                    type_span,
                    "a borrow in a field names one of its type's sources, in backticks",
                    "a struct names the sources of its borrows, as in ``Lend from `text` reading`` (D171)",
                )));
            }
        }
        if backticked {
            if !scope.is_some_and(|scope| scope.names.contains(&source.text)) {
                return Err(self.error(Diagnostic::refused(
                    source.span,
                    format!("`{}` isn't a source here", source.text),
                    UNKNOWN_WHY,
                )));
            }
            return Ok(quoted(&source.text));
        }
        Ok(source.text.clone())
    }

    /// A type that holds borrows fills every source, in the declared order; one that holds none has none to fill
    /// (D171).
    pub fn check_fills(&mut self, written: &syntax::Type, name: &str, sources: &[String]) -> Check<()> {
        let syntax::TypeKind::Named { fill, .. } = &written.kind else { return Ok(()) };
        if sources.is_empty() {
            if fill.is_empty() {
                return Ok(());
            }
            return Err(self.error(Diagnostic::refused(
                written.span,
                format!("`{name}` holds no borrows, so it has no sources to fill"),
                "a struct that holds a borrow names its sources, and others have none (D171)",
            )));
        }
        let listed = sources.iter().map(|source| format!("`{source}`")).collect::<Vec<_>>().join(", then ");
        for (index, source) in sources.iter().enumerate() {
            let Some(argument) = fill.get(index) else {
                return Err(self.error(Diagnostic::refused(
                    written.span,
                    format!("`{name}` has the sources {listed}, and this leaves out `{source}`"),
                    ORDER_WHY,
                )));
            };
            let Some(label) = &argument.label else { return Err(self.not_yet(argument.span, "this fill".to_string())) };
            if label.text != *source {
                if sources.contains(&label.text) {
                    return Err(self.error(Diagnostic::refused(
                        label.span,
                        format!("`{name}`'s sources go in their declared order: {listed}"),
                        "a type fills every source, in the declared order (D171)",
                    )));
                }
                return Err(self.error(Diagnostic::refused(
                    label.span,
                    format!("`{}` isn't a source of `{name}`", label.text),
                    ORDER_WHY,
                )));
            }
            if let syntax::TypeArgumentValue::Value(value) = &argument.value
                && self.slice(value.span).starts_with('`')
                && let syntax::ExpressionKind::Variable(filled) = &value.kind
                && !self.source_scope.as_ref().is_some_and(|scope| scope.names.contains(&filled.text))
            {
                return Err(self.error(Diagnostic::refused(
                    value.span,
                    format!("`{}` isn't a source here", filled.text),
                    UNKNOWN_WHY,
                )));
            }
        }
        Ok(())
    }

    /// What fills each source in a written type, as `(source, fill)`: a variable, `program`, or a source in backticks.
    #[must_use]
    pub fn fills_of(&self, written: &syntax::Type) -> Vec<(String, String)> {
        let syntax::TypeKind::Named { fill, .. } = &written.kind else { return Vec::new() };
        fill.iter()
            .filter_map(|argument| {
                let label = argument.label.as_ref()?;
                let syntax::TypeArgumentValue::Value(value) = &argument.value else { return None };
                let text = self.slice(value.span).to_string();
                Some((label.text.clone(), text))
            })
            .collect()
    }

    /// The sources a struct or union declares, by its type.
    #[must_use]
    pub fn sources_of(&self, value_type: &Type) -> (Vec<String>, Vec<(String, String)>) {
        match value_type {
            Type::Struct { id, .. } => self
                .aggregates
                .structs
                .get(*id)
                .map(|info| (info.sources.clone(), info.facts.clone()))
                .unwrap_or_default(),
            Type::Union { id, .. } => {
                self.aggregates.unions.get(*id).map(|info| (info.sources.clone(), Vec::new())).unwrap_or_default()
            }
            _ => (Vec::new(), Vec::new()),
        }
    }

    /// A `let` of a type that holds borrows: each fill is a variable in scope, the type's `lasts_at_least` facts hold
    /// for them, and the variable becomes a stored borrow of them (D171).
    pub fn let_fills(&mut self, body: &Body, written: &syntax::Type, declared: &Type) -> Check<Vec<(String, String)>> {
        let (sources, facts) = self.sources_of(declared);
        if sources.is_empty() {
            return Ok(Vec::new());
        }
        let fills = self.fills_of(written);
        for (_, fill) in &fills {
            if fill == "program" || fill.starts_with('`') {
                continue;
            }
            if body.lookup(fill).is_none() {
                return Err(self.error(Diagnostic::refused(
                    written.span,
                    format!("`{fill}` isn't declared here"),
                    "a source is filled with a value in scope, `program`, or a source (D171)",
                )));
            }
        }
        let filled = |source: &str| fills.iter().find(|(each, _)| each == source).map(|(_, fill)| fill.clone());
        for (longer, shorter) in &facts {
            let (Some(long_fill), Some(short_fill)) = (filled(longer), filled(shorter)) else { continue };
            if !lasts_at_least(body, &long_fill, &short_fill) {
                let name = declared.to_string();
                return Err(self.error(Diagnostic::refused(
                    written.span,
                    format!(
                        "`{long_fill}` doesn't last as long as `{short_fill}`, and `{name}` says `{longer}` lasts at least as long as `{shorter}`"
                    ),
                    "a struct's `lasts_at_least` fact holds for every type that fills it in (D171)",
                )));
            }
        }
        Ok(fills)
    }

    /// A field that borrows, being built: `words = Lend $article` borrows from what fills its source (D171).
    pub fn built_borrow(
        &mut self,
        body: &mut Body,
        owner: &str,
        build_span: Span,
        value: &syntax::Expression,
        field_type: &Type,
    ) -> Check<crate::checked::Expression> {
        let Type::Borrow { sources, inner, .. } = field_type else { return Err(Reported) };
        let Some(fills) = body.expected_fills.clone() else {
            return Err(self.error(Diagnostic::not_designed(
                build_span,
                format!("building `{owner}`, which holds borrows, where nothing says its sources"),
                NOT_DECIDED,
            )));
        };
        let syntax::ExpressionKind::Owned { ownership: syntax::Ownership::Lend, value: lent, .. } = &value.kind else {
            return Err(self.not_yet(value.span, "a field that borrows, built from this".to_string()));
        };
        let source = sources.first().map(|source| source.trim_matches('`').to_string()).unwrap_or_default();
        let fill = fills.iter().find(|(each, _)| *each == source).map(|(_, fill)| fill.clone()).unwrap_or_default();
        if fill == "program" {
            return Err(self.error(Diagnostic::not_designed(
                value.span,
                "building a borrow whose source is filled with `program`",
                NOT_DECIDED,
            )));
        }
        if let Some(root) = super::expressions::root_name(lent) {
            let originals = match body.lookup(&root.text).map(|variable| variable.value_type.clone()) {
                Some(Type::Borrow { sources: theirs, .. }) => theirs,
                _ => vec![root.text.clone()],
            };
            if !fill.starts_with('`') && !originals.contains(&fill) {
                return Err(self.error(Diagnostic::refused(
                    value.span,
                    format!(
                        "this borrows `{}`, and the type says this field's source is `{fill}`",
                        originals.join(", ")
                    ),
                    "a type fills each source, and every field borrows from what fills its own (D171)",
                )));
            }
        }
        // The field holds a copy of what it borrows: nothing can change that while the borrow is in use, so the copy
        // reads the same (D169).
        let typed = self.expression(body, lent, Some(inner))?;
        self.convert_into(lent.span, typed, inner)
    }
}

/// Whether one variable lasts at least as long as another: it's declared in the same block or one around it, or it's
/// a parameter, which lasts the whole call.
fn lasts_at_least(body: &Body, longer: &str, shorter: &str) -> bool {
    let depth = |name: &str| body.scopes.iter().rposition(|scope| scope.iter().any(|(each, _)| each == name));
    match (depth(longer), depth(shorter)) {
        (Some(long), Some(short)) => long <= short,
        _ => true,
    }
}
