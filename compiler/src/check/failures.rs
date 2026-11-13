//! Failures (standard 8): `fails` lists in headers, `fail`, and how a call's failure is handled, by passing it up
//! with `?`, using a fallback with `??`, handling it in `otherwise:` arms, or, with nothing, stopping the program
//! (D96).
//!
//! A function that can fail returns a result: its value, a union whose case 0 means no failure and whose other cases
//! are its failures with their data, and the place the failure was raised, for the crash message's `because:`.

use std::collections::HashMap;

use crate::checked::{self, ExpressionKind as Kind, MachineType};
use crate::diagnostic::{Diagnostic, Kind as DiagnosticKind};
use crate::source::Span;
use crate::syntax::{self, ExpressionKind};

use super::aggregates::FieldInfo;
use super::expressions::{Typed, When, failure};
use super::types::{IntegerKind, Type};
use super::{Body, Check, Checker, Origin, Reported, State, Variable};

const NOT_DECIDED_FAILURES: &str = "not decided yet (standard 12.6)";

/// One failure a function lists.
#[derive(Clone, Debug)]
pub struct FailureInfo {
    /// The full name, `file_failed.denied`, or the one-off name, `odd`.
    pub full: String,
    /// The last part, which a `case` or `fail` can use alone when it's the only one with it.
    pub short: String,
    pub fields: Vec<FieldInfo>,
}

/// How a failing function's result is laid out: its struct, and the union of its failures.
#[derive(Clone, Copy, Debug)]
pub struct ResultLayout {
    pub structure: usize,
    pub union: usize,
}

/// An `otherwise:` arm being checked, which `instead` gives its value to.
#[derive(Clone, Debug)]
pub struct Handler {
    pub value_type: Type,
    pub result: checked::LocalId,
    /// How many loops were open when the arm started, since `instead` inside a loop in its arm isn't decided.
    pub loops: usize,
    /// How many variables were in scope as the arm started, so `instead` knows which ones it leaves behind.
    pub variables: usize,
}

/// The named sets of failures, `fam.failure` (D47).
#[derive(Default)]
pub struct FailureSets {
    pub sets: HashMap<String, Vec<(String, Vec<FieldInfo>)>>,
}

impl Checker<'_> {
    /// Reads every `fam.failure` (D47).
    pub fn declare_failure_sets(&mut self, file: &syntax::File) {
        for item in &file.items {
            let syntax::ItemKind::Family(family) = &item.kind else { continue };
            if family.keyword.text() != "fam.failure" {
                continue;
            }
            let set = family.name.text();
            let expected = format!("end fam.failure {set}");
            self.check_closer(&family.end, &expected, super::CLOSER_WHY);
            let mut members: Vec<(String, Vec<FieldInfo>)> = Vec::new();
            for entry in &family.entries {
                let written = entry.name.text();
                let Some(member) = written.strip_prefix(&format!("{set}.")).map(str::to_string) else {
                    self.error(Diagnostic::refused_fix(
                        entry.name.span,
                        format!("{set}.{written}"),
                        "a named set's failures say the set's name (D47)",
                    ));
                    continue;
                };
                if member.contains('.') {
                    self.error(Diagnostic::refused(
                        entry.name.span,
                        format!("`{written}` goes more than one level into `{set}`"),
                        "a family is one level deep, so a family can't hold a family (D161)",
                    ));
                    continue;
                }
                if members.iter().any(|(name, _)| *name == member) {
                    self.error(Diagnostic::refused(
                        entry.name.span,
                        format!("`{written}` is in this set twice"),
                        "a named set is a list of failures, each with its own name (D47)",
                    ));
                    continue;
                }
                let fields = match &entry.kind {
                    syntax::FamilyEntryKind::Failure { fields } => self.failure_fields(fields),
                    _ => Vec::new(),
                };
                members.push((member, fields));
            }
            self.failure_sets.sets.insert(set, members);
        }
    }

    fn failure_fields(&mut self, written: &[syntax::Field]) -> Vec<FieldInfo> {
        written
            .iter()
            .map(|field| FieldInfo {
                name: field.name.text.clone(),
                value_type: self.resolve_type(&field.written_type).unwrap_or(Type::Unknown),
                span: field.span,
            })
            .collect()
    }

    /// A header's `fails [ … ]`, read into the failures it lists.
    pub fn fail_list(&mut self, list: &syntax::FailList) -> Check<Vec<FailureInfo>> {
        let mut failures: Vec<FailureInfo> = Vec::new();
        for entry in &list.entries {
            let written = entry.name.text();
            let added: Vec<FailureInfo> = if let Some(members) = self.failure_sets.sets.get(&written) {
                members
                    .iter()
                    .map(|(member, fields)| FailureInfo {
                        full: format!("{written}.{member}"),
                        short: member.clone(),
                        fields: fields.clone(),
                    })
                    .collect()
            } else if let Some((set, member)) = written.split_once('.')
                && let Some(members) = self.failure_sets.sets.get(set)
            {
                let Some((_, fields)) = members.iter().find(|(name, _)| name == member) else {
                    return Err(self.not_yet(entry.name.span, format!("the failure `{written}`")));
                };
                vec![FailureInfo { full: written.clone(), short: member.to_string(), fields: fields.clone() }]
            } else {
                let fields = self.failure_fields(&entry.fields);
                vec![FailureInfo { full: written.clone(), short: written.clone(), fields }]
            };
            for failure in added {
                if failures.iter().any(|earlier| earlier.short == failure.short && earlier.full != failure.full)
                    && !list.entries.iter().any(|each| each.name.text() == failure.full)
                {
                    return Err(self.error(Diagnostic::refused(
                        entry.name.span,
                        format!("two failures here are called `{}`", failure.short),
                        "two sets sharing a failure's name need the full name in a list (D47)",
                    )));
                }
                failures.push(failure);
            }
        }
        Ok(failures)
    }

    /// Makes a failing function's result type: its value, a union of its failures after case 0 for none, and the
    /// place it was raised.
    pub fn result_layout(&mut self, returns: &Type, failures: &[FailureInfo]) -> ResultLayout {
        let union = self.program.unions.len();
        let mut cases = vec![Vec::new()];
        for failure in failures {
            cases.push(failure.fields.iter().map(|field| field.value_type.machine()).collect());
        }
        self.program.unions.push(cases);
        let value = match returns.machine() {
            MachineType::Void => MachineType::Integer { bits: 8, signed: false },
            other => other,
        };
        let structure = self.program.structs.len();
        self.program.structs.push(vec![value, MachineType::Union(union), MachineType::Address]);
        ResultLayout { structure, union }
    }

    /// Finds a failure by its full name, or its short one when only one has it.
    fn find_failure(failures: &[FailureInfo], name: &str) -> Option<usize> {
        failures.iter().position(|failure| failure.full == name).or_else(|| {
            let matching: Vec<usize> = failures
                .iter()
                .enumerate()
                .filter(|(_, failure)| failure.short == name)
                .map(|(index, _)| index)
                .collect();
            if matching.len() == 1 { Some(matching[0]) } else { None }
        })
    }

    /// A value going out of a failing function with no failure: its value, case 0, and no place.
    pub fn succeed(&self, body: &Body, value: Option<checked::Expression>) -> checked::Expression {
        let Some(layout) = body.result else {
            return value.unwrap_or(checked::Expression { kind: Kind::Integer(0), machine_type: MachineType::Void });
        };
        let value_type = match body.returns.machine() {
            MachineType::Void => MachineType::Integer { bits: 8, signed: false },
            other => other,
        };
        let value = value.unwrap_or(checked::Expression { kind: Kind::Integer(0), machine_type: value_type });
        checked::Expression {
            kind: Kind::Build(vec![
                value,
                checked::Expression {
                    kind: Kind::BuildCase { case: 0, fields: Vec::new() },
                    machine_type: MachineType::Union(layout.union),
                },
                checked::Expression { kind: Kind::Zero, machine_type: MachineType::Address },
            ]),
            machine_type: MachineType::Struct(layout.structure),
        }
    }

    /// `fail odd;` or `fail negative[val = $n];`: leaves with the failure (D52).
    pub fn fail_statement(
        &mut self,
        body: &mut Body,
        span: Span,
        value: &syntax::Expression,
        block: &mut checked::Block,
    ) -> Check<()> {
        let (path, labels) = match &value.kind {
            ExpressionKind::Path(path) => (path, &[][..]),
            ExpressionKind::Bracket { base, labels, .. } => match &base.kind {
                ExpressionKind::Path(path) => (path, labels.as_slice()),
                _ => return Err(self.not_yet(value.span, "failing with this".to_string())),
            },
            _ => return Err(self.not_yet(value.span, "failing with this".to_string())),
        };
        let name = path.text();
        let Some(layout) = body.result else {
            return Err(self.error(Diagnostic::refused(
                path.span,
                format!("this function lists no failures, so it can't `fail {name}`"),
                "a function that can fail lists its failures in its body (D52)",
            )));
        };
        let Some(index) = Self::find_failure(&body.failures, &name) else {
            return Err(self.error(Diagnostic::refused(
                path.span,
                format!("`{name}` isn't in this function's `fails`"),
                "a function that can fail lists its failures in its body (D52)",
            )));
        };
        let info = body.failures[index].clone();
        let mut values: Vec<Option<checked::Expression>> = vec![None; info.fields.len()];
        for label in labels {
            let label_name = label.name.text();
            let Some(field) = info.fields.iter().position(|each| each.name == label_name) else {
                return Err(self.error(Diagnostic::refused(
                    label.name.span,
                    format!("`{name}` carries no `{label_name}`"),
                    "a failure can carry data in `[ ]` (D47)",
                )));
            };
            values[field] = Some(self.value_into(body, &label.value, &info.fields[field].value_type)?);
        }
        if let Some(missing) = values.iter().position(Option::is_none) {
            return Err(self.error(Diagnostic::refused(
                span,
                format!("`{name}` needs `{}`", info.fields[missing].name),
                "a failure can carry data in `[ ]` (D47)",
            )));
        }
        let site = body.sites.len();
        body.sites.push(self.site(span, "it was raised here"));
        let value_type = match body.returns.machine() {
            MachineType::Void => MachineType::Integer { bits: 8, signed: false },
            other => other,
        };
        let result = checked::Expression {
            kind: Kind::Build(vec![
                checked::Expression { kind: Kind::Zero, machine_type: value_type },
                checked::Expression {
                    kind: Kind::BuildCase { case: index + 1, fields: values.into_iter().flatten().collect() },
                    machine_type: MachineType::Union(layout.union),
                },
                checked::Expression { kind: Kind::Site(site), machine_type: MachineType::Address },
            ]),
            machine_type: MachineType::Struct(layout.structure),
        };
        block.push(checked::Statement::Return(Some(result)));
        Ok(())
    }

    /// A call to a failing function with nothing handling it: a failure stops the program, saying where it was
    /// raised (D96).
    pub fn crash_on_failure(&mut self, span: Span, signature: &super::Signature, call: checked::Expression) -> Typed {
        let mut messages = Vec::new();
        for (index, info) in signature.failures.iter().enumerate() {
            let carried = info.fields.len() == 1;
            let diagnostic = Diagnostic::new(
                DiagnosticKind::CheckFailed,
                span,
                format!("`{}` failed, and nothing handled it", info.short),
                "a failure nothing handles stops the program (D96)",
            );
            let when = if carried { vec![When::Text("it carried ".into()), When::Operand(0)] } else { Vec::new() };
            messages.push(checked::Unhandled { case: index + 1, failure: failure(self, diagnostic, &when), carried });
        }
        self.handled(signature, call, checked::OnFailure::Crash(messages))
    }

    /// Wraps a failing call with how its failure is handled.
    fn handled(
        &mut self,
        signature: &super::Signature,
        call: checked::Expression,
        on_failure: checked::OnFailure,
    ) -> Typed {
        let layout = signature.result.unwrap_or(ResultLayout { structure: 0, union: 0 });
        let mut typed = Typed::new(
            signature.returns.clone(),
            Kind::Handle { call: Box::new(call), union: layout.union, on_failure: Box::new(on_failure) },
        );
        if signature.returns == Type::Void {
            typed.value.machine_type = MachineType::Void;
        }
        typed
    }

    /// `call()?`, `call() ?? value` and `call() otherwise: … end otherwise` (D44, D45, D47, D139).
    pub fn handle(
        &mut self,
        body: &mut Body,
        expression: &syntax::Expression,
        expected: Option<&Type>,
    ) -> Check<Typed> {
        let inner = match &expression.kind {
            ExpressionKind::Pass { value, .. }
            | ExpressionKind::Instead { value, .. }
            | ExpressionKind::Otherwise { value, .. } => value.as_ref(),
            _ => return Err(Reported),
        };
        let inner = match &inner.kind {
            ExpressionKind::Group(grouped) => grouped.as_ref(),
            _ => inner,
        };
        if let ExpressionKind::MethodCall { receiver, method, arguments, .. } = &inner.kind {
            let typed = self.method_call(body, inner, receiver, method, arguments)?;
            let Kind::Handle { call, .. } = typed.value.kind else {
                return Err(self.not_yet(expression.span, "handling a call that can't fail".to_string()));
            };
            let Kind::Call { function, .. } = &call.kind else { return Err(Reported) };
            let signature =
                self.functions.values().find(|signature| signature.id == *function).cloned().ok_or(Reported)?;
            return self.handle_call(body, expression, expected, signature, *call);
        }
        // Crossing a `type.new` with `as` can't fail (D141), so nothing is caught.
        if let ExpressionKind::As { value, .. } = &inner.kind
            && let Ok(crossed) = self.expression(body, inner, None)
            && let Ok(source) = self.expression(body, value, None)
            && (matches!(crossed.value_type, Type::New { .. }) || matches!(source.value_type, Type::New { .. }))
        {
            let word = match &expression.kind {
                ExpressionKind::Pass { .. } => "?",
                ExpressionKind::Instead { .. } => "??",
                _ => "otherwise",
            };
            return Err(self.error(Diagnostic::not_designed(
                expression.span,
                format!("`{word}` on a call that can't fail"),
                super::NOT_DECIDED,
            )));
        }
        let ExpressionKind::Call { callee, arguments, .. } = &inner.kind else {
            return Err(self.error(Diagnostic::not_designed(
                inner.span,
                "what `?`, `??` or `otherwise` catches on anything but one call",
                "not decided yet, and the standard doesn't say",
            )));
        };
        let (signature, call) = self.raw_call(body, inner, callee, arguments)?;
        if signature.failures.is_empty() {
            return Err(self.not_yet(expression.span, "handling a call that can't fail".to_string()));
        }
        self.handle_call(body, expression, expected, signature, call)
    }

    /// Handles a failing call the way the expression around it says.
    fn handle_call(
        &mut self,
        body: &mut Body,
        expression: &syntax::Expression,
        expected: Option<&Type>,
        signature: super::Signature,
        call: checked::Expression,
    ) -> Check<Typed> {
        match &expression.kind {
            ExpressionKind::Pass { .. } => self.pass_up(body, expression.span, &signature, call),
            ExpressionKind::Instead { fallback, .. } => {
                if matches!(fallback.kind, ExpressionKind::Call { .. }) {
                    return Err(self.error(Diagnostic::not_designed(
                        fallback.span,
                        "whether `??` can take a call, not just a value",
                        NOT_DECIDED_FAILURES,
                    )));
                }
                if let ExpressionKind::Unary { operator: syntax::UnaryOperator::Negate, operand, .. } = &fallback.kind
                    && !matches!(operand.kind, ExpressionKind::Integer(_) | ExpressionKind::Float(_))
                {
                    return Err(self.error(Diagnostic::refused(
                        fallback.span,
                        "`??`'s right side is one value, and unary `-` sits below it, so this needs `( )`",
                        "`??` sticks to its call, and its right side is one value (D132)",
                    )));
                }
                if signature.returns == Type::Void {
                    if !matches!(&fallback.kind, ExpressionKind::Path(path) if path.is_single("void")) {
                        return Err(
                            self.not_yet(fallback.span, "a fallback for a call that gives nothing back".to_string())
                        );
                    }
                    return Ok(self.handled(&signature, call, checked::OnFailure::Ignore));
                }
                let value_type = expected.cloned().unwrap_or_else(|| signature.returns.clone());
                let _ = value_type;
                let mark = body.events.len();
                let fallback_value = self.value_into(body, fallback, &signature.returns);
                body.maybe_since(mark);
                let fallback_value = fallback_value?;
                Ok(self.handled(&signature, call, checked::OnFailure::Fallback(fallback_value)))
            }
            ExpressionKind::Otherwise { value, keyword, handling: syntax::Handling::Arms { arms, end } } => {
                self.check_closer(end, "end otherwise", super::CLOSER_WHY);
                // The arms' opener: from the call to `otherwise:`.
                let call_span = Span { file: value.span.file, start: value.span.start, end: keyword.end + 1 };
                self.arms(body, &signature, call, arms, call_span)
            }
            ExpressionKind::Otherwise { value, keyword, handling: syntax::Handling::Handler(path) } => {
                let arms = self.handler_arms(path)?;
                let call_span = Span { file: value.span.file, start: value.span.start, end: keyword.end };
                self.arms(body, &signature, call, &arms, call_span)
            }
            _ => Err(Reported),
        }
    }

    /// `?`: hands the failure to this function's caller (D44, D47).
    fn pass_up(
        &mut self,
        body: &Body,
        span: Span,
        signature: &super::Signature,
        call: checked::Expression,
    ) -> Check<Typed> {
        if body.is_main {
            return Err(self.error(Diagnostic::not_designed(span, "`?` in `fn.main`", NOT_DECIDED_FAILURES)));
        }
        let Some(layout) = body.result else {
            return Err(self.error(Diagnostic::not_designed(
                span,
                "`?` in a function that lists no failures",
                NOT_DECIDED_FAILURES,
            )));
        };
        let mut mapping = Vec::new();
        for (index, info) in signature.failures.iter().enumerate() {
            let Some(mine) = Self::find_failure(&body.failures, &info.full)
                .or_else(|| Self::find_failure(&body.failures, &info.short))
            else {
                return Err(self.error(Diagnostic::refused(
                    span,
                    format!("this passes up `{}`, which this function doesn't list", info.short),
                    "`?` is a compile error if the function doesn't list what it passes up (D47)",
                )));
            };
            mapping.push((index + 1, mine + 1));
        }
        let returns = match body.returns.machine() {
            MachineType::Void => MachineType::Integer { bits: 8, signed: false },
            other => other,
        };
        Ok(self.handled(
            signature,
            call,
            checked::OnFailure::PassUp {
                mapping,
                structure: layout.structure,
                union: layout.union,
                value_type: returns,
            },
        ))
    }

    /// `otherwise:` arms, which cover every failure the call can have (D45, D139).
    fn arms(
        &mut self,
        body: &mut Body,
        signature: &super::Signature,
        call: checked::Expression,
        arms: &[syntax::Arm],
        call_span: Span,
    ) -> Check<Typed> {
        let layout = signature.result.unwrap_or(ResultLayout { structure: 0, union: 0 });
        let result = body.new_local(signature.returns.machine());
        let union_local = body.new_local(MachineType::Union(layout.union));
        let start = body.states.len();
        let mut checked_arms = Vec::new();
        let mut ends = vec![body.states.clone()];
        let mut failed = false;
        let mut covered = vec![false; signature.failures.len()];
        for arm in arms {
            let name = arm.pattern.text();
            let Some(index) = Self::find_failure(&signature.failures, &name) else {
                self.error(Diagnostic::refused(
                    arm.pattern.span,
                    format!("this call can't fail with `{name}`"),
                    "an arm handles a failure still possible at that call (D46)",
                ));
                failed = true;
                continue;
            };
            covered[index] = true;
            let before = body.states.clone();
            body.scopes.push(Vec::new());
            if let Some(binding) = &arm.binding {
                if body.find(&binding.text).is_some() {
                    self.error(Diagnostic::not_designed(
                        binding.span,
                        format!("declaring `{}` again while the first is in scope", binding.text),
                        "not decided yet, and the standard doesn't say",
                    ));
                    failed = true;
                    body.scopes.pop();
                    continue;
                }
                let fields = signature.failures[index]
                    .fields
                    .iter()
                    .map(|field| (field.name.clone(), field.value_type.clone()))
                    .collect();
                body.declare(
                    Variable {
                        name: binding.text.clone(),
                        value_type: Type::Failure {
                            union: layout.union,
                            name: signature.failures[index].short.clone(),
                            fields,
                        },
                        mutable: false,
                        origin: Origin::Let,
                        local: union_local,
                        declared: binding.span,
                        given: None,
                        known_case: Some(index + 1),
                    },
                    State::Assigned,
                );
            }
            body.handlers.push(Handler {
                value_type: signature.returns.clone(),
                result,
                loops: body.loops.len(),
                variables: body.variables.len(),
            });
            let errors_before = self.errors.len();
            let checked = self.block(body, &arm.body);
            body.handlers.pop();
            body.scopes.pop();
            body.forget_after(start);
            // An arm with its own error already said what's wrong with its end.
            if !checked.leaves && signature.returns != Type::Void && self.errors.len() == errors_before {
                self.error(Diagnostic::refused(
                    arm.pattern.span,
                    format!("this arm can run off its end, and `{}` is needed here", signature.returns),
                    "an arm gives a value with `instead`, or can `return`, `break` or `fail` (D139)",
                ));
                failed = true;
            }
            if !checked.leaves || checked.gives {
                ends.push(body.states.clone());
            }
            body.states.clone_from(&before);
            checked_arms.push((index + 1, checked.block));
        }
        if failed {
            return Err(Reported);
        }
        if let Some(missing) = covered.iter().position(|done| !done) {
            return Err(self.error(Diagnostic::refused(
                call_span,
                format!("this doesn't handle `{}`", signature.failures[missing].short),
                "the arms must cover every failure still possible at that call (D46)",
            )));
        }
        if let Some(first) = ends.first().cloned() {
            body.states = ends.iter().skip(1).fold(first, |joined, each| Body::join_states(&joined, each));
        }
        Ok(self.handled(signature, call, checked::OnFailure::Arms { arms: checked_arms, result, union_local }))
    }

    /// A handler narrowing's arms, used at a call as if written there (D46). A handler that can `return` or `break`
    /// is marked `*` where it's used, and one that can't isn't.
    pub(super) fn handler_arms(&mut self, path: &syntax::Path) -> Check<Vec<syntax::Arm>> {
        let written = path.text();
        let starred = path.last().is_some_and(|segment| segment.mark == syntax::Mark::Star);
        let plain = written.replacen(".*", ".", 1);
        let Some(arms) = self.handlers.get(&plain).cloned() else {
            return Err(self.error(Diagnostic::refused(
                path.span,
                format!("there's no handler `{plain}`"),
                "a handler narrowing is a named set of `otherwise` arms (D46)",
            )));
        };
        let jumps = arms.iter().any(|arm| arm_jumps(&arm.body));
        if jumps && !starred {
            let fixed = match plain.rsplit_once('.') {
                Some((set, name)) => format!("{set}.*{name}"),
                None => plain.clone(),
            };
            return Err(self.error(Diagnostic::refused_fix(
                path.span,
                fixed,
                "a handler that can `return` or `break` is marked `*` where it's used (D46, D128)",
            )));
        }
        if !jumps && starred {
            return Err(self.error(Diagnostic::refused_fix(
                path.span,
                plain,
                "the compiler refuses `*` on a handler that can't jump (D46)",
            )));
        }
        Ok(arms)
    }

    /// Reads every `fam.handler` (D46).
    pub fn declare_handlers(&mut self, file: &syntax::File) {
        for item in &file.items {
            let syntax::ItemKind::Family(family) = &item.kind else { continue };
            if family.keyword.text() != "fam.handler" {
                continue;
            }
            for entry in &family.entries {
                if let syntax::FamilyEntryKind::Handler { arms, .. } = &entry.kind {
                    self.handlers.insert(entry.name.text(), arms.clone());
                }
            }
        }
    }

    /// `instead value;`: the value used in place of the failed call (D139).
    pub fn instead_statement(
        &mut self,
        body: &mut Body,
        span: Span,
        value: &syntax::Expression,
        block: &mut checked::Block,
    ) -> Check<()> {
        let Some(handler) = body.handlers.last().cloned() else {
            return Err(self.error(Diagnostic::refused(
                span,
                "`instead` goes in a failure's `case`, and there's none around this",
                "`instead` gives the value used in place of a failed call (D139)",
            )));
        };
        if body.loops.len() > handler.loops {
            return Err(self.error(Diagnostic::not_designed(
                span,
                "`instead` inside a loop inside its arm",
                super::NOT_DECIDED,
            )));
        }
        if handler.value_type == Type::Void {
            return Err(self.error(Diagnostic::refused(
                span,
                "this call gives back nothing, so there's nothing to use instead",
                "`instead` gives the value used in place of a failed call (D139)",
            )));
        }
        let value = self.value_into(body, value, &handler.value_type)?;
        let start = handler.variables.min(body.variables.len());
        let locals = body.variables[start..].iter().map(|variable| variable.local).collect();
        body.events.push(crate::flow::Event::Leave { span, locals: Some(locals) });
        block.push(checked::Statement::Instead { local: handler.result, value });
        Ok(())
    }

    /// A call's checked form before its failure is handled, with its signature.
    fn raw_call(
        &mut self,
        body: &mut Body,
        expression: &syntax::Expression,
        callee: &syntax::Expression,
        arguments: &[syntax::Argument],
    ) -> Check<(super::Signature, checked::Expression)> {
        let typed = self.call_unhandled(body, expression, callee, arguments)?;
        let Kind::Call { function, .. } = &typed.value.kind else { return Err(Reported) };
        let signature = self.functions.values().find(|signature| signature.id == *function).cloned().ok_or(Reported)?;
        Ok((signature, typed.value))
    }
}

/// The type a failing function's call gives back while unhandled: its result struct.
#[must_use]
pub fn result_machine_type(layout: ResultLayout) -> MachineType {
    MachineType::Struct(layout.structure)
}

/// The machine type of a plain `i32`, for places.
#[must_use]
pub fn place_type() -> MachineType {
    Type::Integer { kind: IntegerKind::I32, bounds: None, unit: None }.machine()
}

/// Whether an arm's code can `return` or `break` out of it.
fn arm_jumps(statements: &[syntax::Statement]) -> bool {
    statements.iter().any(|statement| match &statement.kind {
        syntax::StatementKind::Return(_) | syntax::StatementKind::Break(_) => true,
        syntax::StatementKind::If(chain) => {
            chain.branches.iter().any(|branch| arm_jumps(&branch.body))
                || chain.otherwise.as_ref().is_some_and(|(_, body)| arm_jumps(body))
        }
        _ => false,
    })
}
