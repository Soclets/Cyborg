//! The top of a body (D114, D209): `labels`, `defaults`, `requires` and `ensures`, in that order, then the code.
//! Labels and defaults say how a function is called (D113, D114), and contracts are checked at runtime (D29, D84).

use crate::checked::{self, ExpressionKind as Kind, MachineType};
use crate::diagnostic::{Diagnostic, Kind as DiagnosticKind};
use crate::source::Span;
use crate::syntax::{self, ExpressionKind, StatementKind};

use super::expressions::{When, failure};
use super::types::Type;
use super::{Body, Check, Checker, Reported};

const TOP_WHY: &str = "the top of a body goes `labels`, `defaults`, `requires`, `ensures`, then the code (D114, D209)";
const LABELS_WHY: &str =
    "a declared label must be written, and a parameter with no label is called without one (D113, D114)";

/// How a function is called and what it promises, from the top of its body.
#[derive(Clone, Debug, Default)]
pub struct TopOfBody {
    /// Each parameter's label, if it has one.
    pub labels: Vec<Option<String>>,
    pub any_order: bool,
    /// Each parameter's default, if it has one.
    pub defaults: Vec<Option<syntax::Expression>>,
    /// The `requires` and `ensures` lines, with whether each is `.only_in_working`.
    pub requires: Vec<(syntax::Expression, bool)>,
    pub ensures: Vec<(syntax::Expression, bool)>,
    /// Each label's span, for errors about a unit it carries (D144).
    pub label_spans: Vec<Option<Span>>,
    /// The unit each label carries, as in `labels [$x as at.screen.pixels];` (D144).
    pub label_units: Vec<Option<super::types::Unit>>,
    /// `requires $list lasts_at_least $article;`: facts about how long parameters last, which every call meets and
    /// nothing checks while running (D171).
    pub lasts: Vec<(String, String)>,
}

impl Checker<'_> {
    /// Reads the top of a function's body, refusing anything out of its order.
    pub fn top_of_body(&mut self, function: &syntax::Function) -> TopOfBody {
        let count = function.parameters.list.len();
        let mut top = TopOfBody { labels: vec![None; count], defaults: vec![None; count], ..TopOfBody::default() };
        let parameter_index =
            |name: &str| function.parameters.list.iter().position(|parameter| parameter.name.text == name);
        let mut rank = 0;
        let mut code_started = false;
        for statement in &function.body {
            let (this_rank, word) = match &statement.kind {
                StatementKind::Labels(_) => (1, "labels"),
                StatementKind::Defaults(_) => (2, "defaults"),
                StatementKind::Requires { .. } => (3, "requires"),
                StatementKind::Ensures { .. } => (4, "ensures"),
                _ => {
                    code_started = true;
                    continue;
                }
            };
            if code_started {
                let span =
                    Span::new(self.file, statement.span.start as usize, statement.span.start as usize + word.len());
                self.error(Diagnostic::refused(span, format!("`{word}` goes first in the body"), TOP_WHY));
                continue;
            }
            if this_rank < rank {
                let span =
                    Span::new(self.file, statement.span.start as usize, statement.span.start as usize + word.len());
                self.error(Diagnostic::refused(span, format!("`{word}` is out of order"), TOP_WHY));
                continue;
            }
            rank = this_rank;
            match &statement.kind {
                StatementKind::Labels(labels) => {
                    let keyword = self.slice(statement.span).split([' ', '[']).next().unwrap_or("").to_string();
                    if keyword == "labels.in_order" {
                        let span = Span::new(
                            self.file,
                            statement.span.start as usize,
                            statement.span.start as usize + keyword.len(),
                        );
                        self.error(Diagnostic::refused_fix(
                            span,
                            "labels",
                            "a default the language fixes can't be written out (D51, D114)",
                        ));
                        continue;
                    }
                    top.any_order = keyword == "labels.any_order";
                    for label in labels {
                        // `$b as to`
                        let ExpressionKind::As { value, written_type, .. } = &label.value.kind else { continue };
                        let ExpressionKind::Variable(name) = &value.kind else { continue };
                        let syntax::TypeKind::Named { path, .. } = &written_type.kind else { continue };
                        if let Some(index) = parameter_index(&name.text) {
                            top.label_spans.resize(count, None);
                            top.label_spans[index] = Some(path.span);
                            top.labels[index] = Some(path.text());
                        }
                    }
                }
                StatementKind::Defaults(defaults) => {
                    for default in defaults {
                        let Some(target) = default.name.segments.first() else { continue };
                        let Some(index) = parameter_index(&target.name.text) else { continue };
                        if top.labels[index].is_none() {
                            self.error(Diagnostic::refused(
                                target.name.span,
                                format!("`{}` has no label, so it can't have a default", target.name.text),
                                "only labelled parameters can have defaults (D114)",
                            ));
                            continue;
                        }
                        top.defaults[index] = Some(default.value.clone());
                    }
                }
                StatementKind::Requires { condition, .. }
                    if let ExpressionKind::Binary {
                        operator: syntax::BinaryOperator::LastsAtLeast,
                        left,
                        right,
                        ..
                    } = &condition.kind =>
                {
                    let longer = self.slice(left.span).to_string();
                    let shorter = self.slice(right.span).to_string();
                    if top.lasts.contains(&(longer.clone(), shorter.clone())) {
                        self.error(Diagnostic::refused(
                            statement.span,
                            format!("`{longer}` lasts at least as long as `{shorter}` is already known"),
                            "the same `lasts_at_least` fact isn't written twice (D171)",
                        ));
                        continue;
                    }
                    top.lasts.push((longer, shorter));
                }
                StatementKind::Requires { keyword, condition } => {
                    top.requires.push((condition.clone(), keyword.text().ends_with(".only_in_working")));
                }
                StatementKind::Ensures { keyword, condition } => {
                    top.ensures.push((condition.clone(), keyword.text().ends_with(".only_in_working")));
                }
                _ => {}
            }
        }
        top
    }

    /// Matches a call's arguments to the parameters by position and label (D113, D114), giving each parameter's
    /// argument, or its default.
    pub fn match_arguments<'a>(
        &mut self,
        call_span: Span,
        signature: &super::Signature,
        arguments: &'a [syntax::Argument],
    ) -> Check<Vec<Matched<'a>>> {
        let count = signature.parameters.len();
        let mut matched: Vec<Option<Matched<'a>>> = vec![None; count];
        let mut positional = 0;
        let mut last_labelled = None;
        for argument in arguments {
            match &argument.label {
                Some(label) => {
                    let Some(index) =
                        signature.top.labels.iter().position(|each| each.as_deref() == Some(label.text.as_str()))
                    else {
                        return Err(self.error(Diagnostic::refused(
                            label.span,
                            format!("`{}` has no label `{}`", signature.name, label.text),
                            LABELS_WHY,
                        )));
                    };
                    if !signature.top.any_order && last_labelled.is_some_and(|last| index < last) {
                        return Err(self.error(Diagnostic::refused(
                            label.span,
                            format!("`{}` is out of order", label.text),
                            "labels keep their declared order, unless declared `labels.any_order` (D114)",
                        )));
                    }
                    last_labelled = Some(index);
                    matched[index] = Some(Matched::Argument(argument));
                }
                None => {
                    // The next parameter in order takes it, and one with a label has to be written with it.
                    while positional < count && matched[positional].is_some() {
                        positional += 1;
                    }
                    if positional >= count {
                        return Err(self.not_yet(argument.span, "more arguments than parameters".to_string()));
                    }
                    if let Some(label) = &signature.top.labels[positional] {
                        let shown = self.slice(argument.span).to_string();
                        return Err(self.error(Diagnostic::refused_fix(
                            argument.span,
                            format!("{label}: {shown}"),
                            LABELS_WHY,
                        )));
                    }
                    matched[positional] = Some(Matched::Argument(argument));
                    positional += 1;
                }
            }
        }
        let mut result = Vec::with_capacity(count);
        for (index, slot) in matched.into_iter().enumerate() {
            match slot {
                Some(found) => result.push(found),
                None => match (&signature.top.defaults[index], &signature.top.labels[index]) {
                    (Some(default), _) => result.push(Matched::Default(default.clone())),
                    (None, Some(label)) => {
                        return Err(self.error(Diagnostic::refused(
                            call_span,
                            format!("this call to `{}` leaves off `{label}:`", signature.name),
                            LABELS_WHY,
                        )));
                    }
                    (None, None) => {
                        return Err(self.not_yet(call_span, "a call with fewer arguments than parameters".to_string()));
                    }
                },
            }
        }
        Ok(result)
    }

    /// The `requires` lines, checked when the function starts, with the call site from its hidden argument (D29).
    pub fn requires_checks(&mut self, body: &mut Body, top: &TopOfBody, site: checked::LocalId) -> checked::Block {
        let mut block = Vec::new();
        for (condition, only_in_working) in &top.requires {
            if *only_in_working && !matches!(self.mode, crate::target::Mode::Working(_)) {
                continue;
            }
            let Ok(checked) = self.contract(body, condition, "requires", None) else { continue };
            block.push(checked::Statement::CheckAtSite {
                condition: checked.0,
                failure: checked.1,
                operands: checked.2,
                site: checked::Expression { kind: Kind::Local(site), machine_type: MachineType::Address },
            });
        }
        block
    }

    /// The `ensures` lines, checked at a `return` with the value it gives back (D29).
    pub fn ensures_checks(
        &mut self,
        body: &mut Body,
        top: &TopOfBody,
        result: checked::LocalId,
        at: Span,
    ) -> checked::Block {
        let mut block = Vec::new();
        for (condition, only_in_working) in &top.ensures {
            if *only_in_working && !matches!(self.mode, crate::target::Mode::Working(_)) {
                continue;
            }
            let Ok((condition, mut failure, operands)) = self.contract(body, condition, "ensures", Some(result)) else {
                continue;
            };
            failure.after.push_str(&crate::diagnostic::render_because(self.sources, at, "this `return` led here"));
            failure.also.push((at, "this `return` led here".to_string()));
            block.push(checked::Statement::Check { condition, failure, operands });
        }
        block
    }

    /// A contract's condition, its failure, and the values its `when:` line prints: every variable it reads, then
    /// `result`.
    fn contract(
        &mut self,
        body: &mut Body,
        condition: &syntax::Expression,
        word: &str,
        result: Option<checked::LocalId>,
    ) -> Check<(checked::Expression, checked::CheckFailure, Vec<checked::Expression>)> {
        body.result_value = result.map(|local| (local, body.returns.clone()));
        body.in_contract = true;
        let typed = self.condition_value(body, condition);
        body.in_contract = false;
        body.result_value = None;
        let typed = typed?;
        let mut names = Vec::new();
        variables_in(condition, &mut names);
        let mut when = Vec::new();
        let mut operands = Vec::new();
        for name in &names {
            let Some(variable) = body.lookup(name).cloned() else { continue };
            if !when.is_empty() {
                when.push(When::Text(", ".into()));
            }
            when.push(When::Text(format!("`{name}` is ")));
            when.push(When::Operand(operands.len()));
            operands.push(checked::Expression {
                kind: Kind::Local(variable.local),
                machine_type: variable.value_type.machine(),
            });
        }
        if let Some(local) = result
            && mentions_result(condition)
        {
            if !when.is_empty() {
                when.push(When::Text(", ".into()));
            }
            when.push(When::Text("`result` is ".into()));
            when.push(When::Operand(operands.len()));
            operands.push(checked::Expression { kind: Kind::Local(local), machine_type: body.returns.machine() });
        }
        let failure = failure(
            self,
            Diagnostic::new(
                DiagnosticKind::CheckFailed,
                condition.span,
                format!("this `{word}` is false"),
                "contracts are checked at runtime, and a false one stops the program (D29)",
            ),
            &when,
        );
        let _ = Type::Void;
        Ok((typed.value, failure, operands))
    }
}

/// A parameter's argument, or its default.
#[derive(Clone, Debug)]
pub enum Matched<'a> {
    Argument(&'a syntax::Argument),
    Default(syntax::Expression),
}

/// The variables an expression reads, in the order they first appear.
fn variables_in(expression: &syntax::Expression, names: &mut Vec<String>) {
    match &expression.kind {
        ExpressionKind::Variable(name) => {
            if !names.contains(&name.text) {
                names.push(name.text.clone());
            }
        }
        ExpressionKind::Unary { operand, .. } | ExpressionKind::Group(operand) => variables_in(operand, names),
        ExpressionKind::Binary { left, right, .. } => {
            variables_in(left, names);
            variables_in(right, names);
        }
        ExpressionKind::Chain { operands, .. } => {
            for operand in operands {
                variables_in(operand, names);
            }
        }
        ExpressionKind::Call { arguments, .. } => {
            for argument in arguments {
                variables_in(&argument.value, names);
            }
        }
        ExpressionKind::Owned { value, .. } => variables_in(value, names),
        _ => {}
    }
}

fn mentions_result(expression: &syntax::Expression) -> bool {
    match &expression.kind {
        ExpressionKind::Result => true,
        ExpressionKind::Unary { operand, .. } | ExpressionKind::Group(operand) => mentions_result(operand),
        ExpressionKind::Binary { left, right, .. } => mentions_result(left) || mentions_result(right),
        ExpressionKind::Chain { operands, .. } => operands.iter().any(mentions_result),
        _ => false,
    }
}

/// Whether a function needs its call site passed in, for its `requires` crash message.
#[must_use]
pub fn needs_site(top: &TopOfBody) -> bool {
    !top.requires.is_empty()
}

/// The `Reported` type, so callers can name it.
pub type Failed = Reported;
