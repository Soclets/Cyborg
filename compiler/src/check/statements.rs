//! Statements, and how variables' states flow through blocks, branches and loops (D203).

use crate::checked::{self, ExpressionKind as Kind, MachineType};
use crate::diagnostic::{Diagnostic, Kind as DiagnosticKind};
use crate::source::Span;
use crate::syntax::{self, Condition, ExpressionKind, StatementKind};

use super::expressions::{CONVERT_WHY, Typed, When, failure};
use super::types::Type;
use super::{Body, Check, Checker, Loop, Origin, Reported, State, Variable};

const RANGE_ORDER_WHY: &str =
    "`from:` is required, `to:` or `before:` says where it ends, and `step_by:` comes after them, in that order (D128)";
const NOT_DECIDED: &str = "not decided yet, and the standard doesn't say";

/// A checked block, and whether every path through it leaves: by `return`, `break`, or a loop that never ends.
pub struct Checked {
    pub block: checked::Block,
    pub leaves: bool,
}

impl Checker<'_> {
    /// A function's whole body.
    pub fn function_block(&mut self, body: &mut Body, statements: &[syntax::Statement]) -> (checked::Block, bool) {
        let checked = self.block(body, statements);
        (checked.block, checked.leaves)
    }

    /// Checks a block in a new scope. A statement with an error is skipped, and checking goes on (standard 11.7).
    pub fn block(&mut self, body: &mut Body, statements: &[syntax::Statement]) -> Checked {
        body.scopes.push(Vec::new());
        let count = body.variables.len();
        let mut block = Vec::new();
        let mut leaves = false;
        for statement in statements {
            body.statement = None;
            let result = self.statement(body, statement, &mut block);
            // A `return` or `break` leaves even when what it hands back is refused.
            if matches!(result, Ok(true))
                || matches!(statement.kind, StatementKind::Return(_) | StatementKind::Break(_))
            {
                leaves = true;
            }
        }
        body.scopes.pop();
        body.forget_after(count);
        Checked { block, leaves }
    }

    /// Checks one statement, adding what it runs to `block`, and gives whether it always leaves.
    fn statement(&mut self, body: &mut Body, statement: &syntax::Statement, block: &mut checked::Block) -> Check<bool> {
        match &statement.kind {
            StatementKind::Let(binding) => self.let_statement(body, binding, block).map(|()| false),
            StatementKind::Decl { mutable, name, written_type } => {
                let value_type = self.resolve_type(written_type).unwrap_or(Type::Unknown);
                let local = body.new_local(value_type.machine());
                body.declare(
                    Variable {
                        name: name.text.clone(),
                        value_type,
                        mutable: *mutable,
                        origin: Origin::Decl,
                        local,
                        declared: name.span,
                        given: None,
                    },
                    State::Unassigned,
                );
                Ok(false)
            }
            StatementKind::Assign { target, value } => {
                self.assign(body, statement.span, target, value, block).map(|()| false)
            }
            StatementKind::Change { target, value } => self.change(body, target, value, block).map(|()| false),
            StatementKind::Return(value) => {
                self.return_statement(body, statement, value.as_ref(), block).map(|()| true)
            }
            StatementKind::Expression(expression) => {
                body.statement = Some(expression.span);
                self.expression_statement(body, expression, block).map(|()| false)
            }
            StatementKind::If(chain) => self.if_statement(body, chain, block),
            StatementKind::While { label, condition, body: inner, end } => {
                if label.is_none() {
                    self.check_closer(end, "end while", super::CLOSER_WHY);
                }
                self.while_statement(body, label.as_ref(), condition, inner, block).map(|()| false)
            }
            StatementKind::Forever { label, body: inner, end } => {
                if label.is_none() {
                    self.check_closer(end, "end forever", super::CLOSER_WHY);
                }
                self.forever(body, label.as_ref(), inner, block)
            }
            StatementKind::For(each) => {
                match &each.label {
                    Some(label) => self.check_closer(&each.end, &format!("end for {}", label.text), super::CLOSER_WHY),
                    None => self.check_closer(
                        &each.end,
                        "end for",
                        "the closer repeats what it closes, including the name (D33, D128)",
                    ),
                }
                self.for_statement(body, each, block).map(|()| false)
            }
            StatementKind::Break(label) => {
                self.break_statement(body, statement.span, label.as_ref(), block).map(|()| true)
            }
            StatementKind::Broken => Err(Reported),
            _ => Err(self.not_yet(statement.span, "this statement".to_string())),
        }
    }

    fn let_statement(&mut self, body: &mut Body, binding: &syntax::Let, block: &mut checked::Block) -> Check<()> {
        let declared = match &binding.written_type {
            Some(written) => self.resolve_type(written).unwrap_or(Type::Unknown),
            None => Type::Unknown,
        };
        let local = body.new_local(declared.machine());
        let result = self.let_value(body, &declared, binding, local, block);
        // The variable exists even if its value is refused, and reading it then is its own error (standard 11.7).
        let origin = if result.is_ok() { Origin::Let } else { Origin::RefusedLet };
        let state = if result.is_ok() { State::Assigned } else { State::Unassigned };
        body.declare(
            Variable {
                name: binding.name.text.clone(),
                value_type: declared,
                mutable: binding.mutable,
                origin,
                local,
                declared: binding.name.span,
                given: None,
            },
            state,
        );
        result
    }

    fn let_value(
        &mut self,
        body: &mut Body,
        declared: &Type,
        binding: &syntax::Let,
        local: checked::LocalId,
        block: &mut checked::Block,
    ) -> Check<()> {
        if declared.is_unknown() {
            // The type was refused, so its value can't be judged against it.
            return Err(Reported);
        }
        if let Type::Writer(writer) = declared {
            let statement = self.open_writer(body, *writer, &binding.value, local)?;
            block.push(statement);
            return Ok(());
        }
        let value = self.value_into(body, &binding.value, declared)?;
        block.push(checked::Statement::Let { local, value });
        Ok(())
    }

    /// `assign $x = …;`: a `decl`'s value, once on each path (D203).
    fn assign(
        &mut self,
        body: &mut Body,
        span: Span,
        target: &syntax::Expression,
        value: &syntax::Expression,
        block: &mut checked::Block,
    ) -> Check<()> {
        let ExpressionKind::Variable(name) = &target.kind else {
            return Err(self.not_yet(target.span, "`assign` to this".to_string()));
        };
        let Some(index) = body.find(&name.text) else {
            return Err(self.error(Diagnostic::refused(
                name.span,
                format!("`{}` isn't declared", name.text),
                "`decl` declares a variable with no value yet, and `assign` gives it one (D203)",
            )));
        };
        let variable = body.variables[index].clone();
        if variable.origin != Origin::Decl || body.states[index] == State::Assigned {
            let written = self.slice(span).to_string();
            let fixed = written.replacen("assign", "change", 1);
            return Err(self.error(Diagnostic::refused_fix(
                span,
                fixed,
                "`assign` gives a `decl` its first value, and `change` gives a new one (D203)",
            )));
        }
        if body.states[index] == State::Maybe {
            return Err(self.error(Diagnostic::refused(
                span,
                format!("`{}` may already have its value here", name.text),
                "`assign` gives a variable its value once on each path (D203)",
            )));
        }
        let value = self.value_into(body, value, &variable.value_type)?;
        block.push(checked::Statement::Let { local: variable.local, value });
        body.states[index] = State::Assigned;
        Ok(())
    }

    /// `change $x = …;` (D3, D125).
    fn change(
        &mut self,
        body: &mut Body,
        target: &syntax::Expression,
        value: &syntax::Expression,
        block: &mut checked::Block,
    ) -> Check<()> {
        let ExpressionKind::Variable(name) = &target.kind else {
            return Err(self.not_yet(target.span, "`change` on this".to_string()));
        };
        let Some(index) = body.find(&name.text) else {
            return Err(self.error(Diagnostic::refused(
                name.span,
                format!("`{}` isn't declared", name.text),
                "a variable is declared with `let`, `let mut` or `decl` before it's used (D203)",
            )));
        };
        let variable = body.variables[index].clone();
        if variable.origin == Origin::Decl && body.states[index] != State::Assigned {
            let what = if body.states[index] == State::Maybe {
                format!("`{}` has no value yet on some path here", name.text)
            } else {
                format!("`{}` has no value yet", name.text)
            };
            return Err(self.error(
                Diagnostic::refused(name.span, what, "every path has to `assign` a `decl` before reading it (D203)")
                    .with_because(variable.declared, format!("`{}` is declared here", name.text)),
            ));
        }
        if !variable.mutable {
            if variable.origin == Origin::Parameter && variable.value_type.is_copied() {
                return Err(self.error(Diagnostic::refused(
                    name.span,
                    format!("`{}` is a plain parameter, so it can't change", name.text),
                    "a plain number parameter can't be changed, so a function can't quietly change its own copy (D184)",
                )));
            }
            return Err(self.error(Diagnostic::refused(
                name.span,
                format!("`{}` can't change", name.text),
                "`let` declares a variable that can't change, and `let mut` one that can (D3)",
            )));
        }
        let value = self.value_into(body, value, &variable.value_type)?;
        block.push(checked::Statement::Set { local: variable.local, value });
        body.states[index] = State::Assigned;
        Ok(())
    }

    fn return_statement(
        &mut self,
        body: &mut Body,
        statement: &syntax::Statement,
        value: Option<&syntax::Expression>,
        block: &mut checked::Block,
    ) -> Check<()> {
        let returns = body.returns.clone();
        let result = match value {
            Some(expression) if matches!(&expression.kind, ExpressionKind::Path(path) if path.is_single("void")) => {
                if returns != Type::Void && !returns.is_unknown() {
                    return Err(self.error(Diagnostic::refused(
                        expression.span,
                        format!("this function returns `{returns}`, so it can't `return void`"),
                        "`return` hands back a value (D11)",
                    )));
                }
                Ok(checked::Statement::Return(None))
            }
            Some(expression) if returns == Type::Void => {
                if body.is_main {
                    return Err(self.not_yet(expression.span, "a value returned from `fn.main`".to_string()));
                }
                Err(self.error(Diagnostic::refused_fix(
                    expression.span,
                    "void",
                    "a function that returns nothing returns `void` (D11)",
                )))
            }
            Some(expression) => {
                body.statement = Some(expression.span);
                self.value_into(body, expression, &returns).map(|value| checked::Statement::Return(Some(value)))
            }
            None => Err(self.not_yet(statement.span, "`return` with nothing after it".to_string())),
        };
        let leaving = self.check_must_give_on_leaving(body, statement.span);
        block.push(result?);
        leaving
    }

    /// Every `must_Give` value still held when leaving is refused (D129, D273).
    fn check_must_give_on_leaving(&mut self, body: &Body, at: Span) -> Check<()> {
        let mut held: Vec<Variable> = Vec::new();
        for scope in &body.scopes {
            for (_, index) in scope {
                let variable = &body.variables[*index];
                if matches!(variable.value_type, Type::Writer(writer) if writer.buffered) && variable.given.is_none() {
                    held.push(variable.clone());
                }
            }
        }
        held.sort_by_key(|variable| variable.declared);
        let mut result = Ok(());
        for variable in held {
            let name = &variable.name;
            result = Err(self.error(
                Diagnostic::refused(
                    at,
                    format!("this leaves with `{name}` never given away or finished"),
                    format!(
                        "a buffered writer is `must_Give`, and `os::done(Give {name});` flushes it for the last time and ends its duty, so no output is lost (D272, D273)"
                    ),
                )
                .with_because(variable.declared, format!("`{name}` was made here")),
            ));
        }
        result
    }

    fn expression_statement(
        &mut self,
        body: &mut Body,
        expression: &syntax::Expression,
        block: &mut checked::Block,
    ) -> Check<()> {
        if let Some(statement) = self.library_statement(body, expression)? {
            block.push(statement);
            return Ok(());
        }
        let typed = self.expression(body, expression, None)?;
        match &typed.value_type {
            Type::Void | Type::Unknown => {}
            other => {
                return Err(self.error(Diagnostic::not_designed(
                    expression.span,
                    format!("dropping a value of `{other}` by not using it"),
                    NOT_DECIDED,
                )));
            }
        }
        block.push(checked::Statement::Evaluate(typed.value));
        Ok(())
    }

    /// An `if` chain, with `else if` and `else:` (D126), and the test (D9, D20).
    fn if_statement(&mut self, body: &mut Body, chain: &syntax::If, block: &mut checked::Block) -> Check<bool> {
        self.check_closer(&chain.end, "end if", super::CLOSER_WHY);
        let start = body.states.len();
        let mut ends: Vec<Vec<State>> = Vec::new();
        let mut branches = Vec::new();
        // Each branch starts from the path where every condition before it was false, which changes nothing
        // variables know, since conditions only read.
        for branch in &chain.branches {
            let condition = self.branch_condition(body, &branch.condition);
            let before = body.states.clone();
            body.scopes.push(Vec::new());
            if let Ok((_, Some(narrowed))) = &condition {
                body.declare(narrowed.clone(), State::Assigned);
            }
            let checked = self.block(body, &branch.body);
            body.scopes.pop();
            body.forget_after(start);
            if !checked.leaves {
                ends.push(body.states[..start].to_vec());
            }
            body.states.clone_from(&before);
            branches.push((condition.map(|(value, _)| value), checked.block));
        }
        let otherwise = match &chain.otherwise {
            Some((_, statements)) => {
                let before = body.states.clone();
                let checked = self.block(body, statements);
                if !checked.leaves {
                    ends.push(body.states[..start].to_vec());
                }
                body.states.clone_from(&before);
                checked.block
            }
            None => {
                ends.push(body.states.clone());
                Vec::new()
            }
        };
        let all_leave = ends.is_empty();
        if let Some(first) = ends.first().cloned() {
            body.states = ends.iter().skip(1).fold(first, |joined, each| Body::join_states(&joined, each));
        }
        let mut built = otherwise;
        let mut failed = false;
        for (condition, then) in branches.into_iter().rev() {
            match condition {
                Ok(condition) => built = vec![checked::Statement::If { condition, then, otherwise: built }],
                Err(Reported) => failed = true,
            }
        }
        if failed {
            return Err(Reported);
        }
        block.extend(built);
        Ok(all_leave)
    }

    /// A branch's condition: a `bool`, or the test on a bound, which narrows the variable inside (D20).
    fn branch_condition(
        &mut self,
        body: &mut Body,
        condition: &Condition,
    ) -> Check<(checked::Expression, Option<Variable>)> {
        match condition {
            Condition::Plain(expression) => Ok((self.condition_value(body, expression)?.value, None)),
            Condition::Test { subject, written_type, binding } => {
                if let Some(binding) = binding {
                    return Err(self.not_yet(binding.span, "a name on the test".to_string()));
                }
                let ExpressionKind::Variable(name) = &subject.kind else {
                    return Err(self.not_yet(subject.span, "the test on something that isn't a variable".to_string()));
                };
                let typed = self.expression(body, subject, None)?;
                let tested = self.resolve_type(written_type)?;
                match (&typed.value_type, &tested) {
                    (Type::Integer { kind: from, .. }, Type::Integer { kind: to, bounds: Some(bounds) })
                        if from == to =>
                    {
                        let machine_type = typed.value.machine_type.clone();
                        let bound = |value: i128| checked::Expression {
                            kind: Kind::Integer(value),
                            machine_type: machine_type.clone(),
                        };
                        let low = compare(checked::Comparison::AtLeast, typed.value.clone(), bound(bounds.min));
                        let high = compare(checked::Comparison::AtMost, typed.value.clone(), bound(bounds.max));
                        let condition = checked::Expression {
                            kind: Kind::And(Box::new(low), Box::new(high)),
                            machine_type: MachineType::Bool,
                        };
                        let Some(original) = body.lookup(&name.text).cloned() else { return Err(Reported) };
                        let narrowed = Variable { value_type: tested.clone(), mutable: false, ..original };
                        Ok((condition, Some(narrowed)))
                    }
                    (
                        Type::Float { bits: from_bits, kind: from_kind, bounds: None },
                        Type::Float { bits, kind, bounds: Some(bounds) },
                    ) if from_bits == bits && from_kind == kind => {
                        let machine_type = typed.value.machine_type.clone();
                        let bound = |value: f64| checked::Expression {
                            kind: Kind::Float(value),
                            machine_type: machine_type.clone(),
                        };
                        let low = compare(checked::Comparison::AtLeast, typed.value.clone(), bound(bounds.min()));
                        let high = compare(checked::Comparison::AtMost, typed.value.clone(), bound(bounds.max()));
                        let condition = checked::Expression {
                            kind: Kind::And(Box::new(low), Box::new(high)),
                            machine_type: MachineType::Bool,
                        };
                        let Some(original) = body.lookup(&name.text).cloned() else { return Err(Reported) };
                        let narrowed = Variable { value_type: tested.clone(), mutable: false, ..original };
                        Ok((condition, Some(narrowed)))
                    }
                    (Type::Float { .. } | Type::Integer { .. }, Type::Float { .. } | Type::Integer { .. }) => Err(self
                        .error(Diagnostic::not_designed(
                            written_type.span,
                            format!("the test for `{tested}` on `{}`", typed.value_type),
                            "not decided yet (standard 12.2)",
                        ))),
                    _ => Err(self.not_yet(subject.span.to(written_type.span), "this test".to_string())),
                }
            }
        }
    }

    fn while_statement(
        &mut self,
        body: &mut Body,
        label: Option<&syntax::Name>,
        condition: &Condition,
        inner: &[syntax::Statement],
        block: &mut checked::Block,
    ) -> Check<()> {
        if let Some(label) = label {
            return Err(self.error(Diagnostic::not_designed(
                label.span,
                "how `while` is named, which 4.8 only shows on `for`",
                NOT_DECIDED,
            )));
        }
        let condition = match condition {
            Condition::Plain(expression) => self.condition_value(body, expression)?.value,
            Condition::Test { subject, written_type, .. } => {
                return Err(self.error(Diagnostic::not_designed(
                    subject.span.to(written_type.span),
                    "the test in a `while`",
                    "not decided yet (standard 12.4)",
                )));
            }
        };
        let before = body.states.clone();
        let (id, loop_body, breaks) = self.loop_body(body, None, inner);
        // The body may not have run at all, or may have stopped at a `break`.
        let mut joined = Body::join_states(&before, &body.states[..before.len()]);
        for each in breaks {
            joined = Body::join_states(&joined, &each[..before.len()]);
        }
        body.states = joined;
        let mut statements = vec![checked::Statement::If {
            condition: not(condition),
            then: vec![checked::Statement::Break(id)],
            otherwise: Vec::new(),
        }];
        statements.extend(loop_body);
        block.push(checked::Statement::Loop { id, body: statements });
        Ok(())
    }

    /// Checks a loop's body, giving its id, its code and the states at each `break` out of it.
    fn loop_body(
        &mut self,
        body: &mut Body,
        label: Option<&syntax::Name>,
        inner: &[syntax::Statement],
    ) -> (checked::LoopId, checked::Block, Vec<Vec<State>>) {
        let id = body.next_loop;
        body.next_loop += 1;
        body.loops.push(Loop { id, label: label.map(|name| name.text.clone()), breaks: Some(Vec::new()) });
        // A body runs again with what it changed, so it's checked from the states it could start a round with:
        // before the loop, joined with how a round ends. The first pass finds those, quietly.
        let before = body.states.clone();
        let locals = body.locals.len();
        let next_loop = body.next_loop;
        self.quiet(|checker| checker.block(body, inner));
        body.locals.truncate(locals);
        body.next_loop = next_loop;
        body.states = Body::join_states(&before, &body.states[..before.len()]);
        if let Some(current) = body.loops.last_mut() {
            current.breaks = Some(Vec::new());
        }
        let checked = self.block(body, inner);
        let breaks = body.loops.pop().and_then(|finished| finished.breaks).unwrap_or_default();
        (id, checked.block, breaks)
    }

    /// Runs a check without keeping its errors, for a first pass whose errors the second pass reports.
    fn quiet<T>(&mut self, run: impl FnOnce(&mut Self) -> T) -> T {
        let errors = self.errors.len();
        let result = run(self);
        self.errors.truncate(errors);
        result
    }

    fn forever(
        &mut self,
        body: &mut Body,
        label: Option<&syntax::Name>,
        inner: &[syntax::Statement],
        block: &mut checked::Block,
    ) -> Check<bool> {
        if let Some(label) = label {
            return Err(self.error(Diagnostic::not_designed(
                label.span,
                "how `forever` is named, which 4.8 only shows on `for`",
                NOT_DECIDED,
            )));
        }
        let before = body.states.len();
        let (id, loop_body, breaks) = self.loop_body(body, None, inner);
        // A `forever` ends only through a `break`, so what follows it starts from the breaks' states (D128).
        let leaves = breaks.is_empty();
        if let Some(first) = breaks.first() {
            let mut joined = first[..before].to_vec();
            for each in &breaks[1..] {
                joined = Body::join_states(&joined, &each[..before]);
            }
            body.states = joined;
        }
        block.push(checked::Statement::Loop { id, body: loop_body });
        Ok(leaves)
    }

    fn break_statement(
        &mut self,
        body: &mut Body,
        span: Span,
        label: Option<&syntax::Name>,
        block: &mut checked::Block,
    ) -> Check<()> {
        if let Some(label) = label
            && super::clashes(&label.text)
        {
            return Err(self.error(Diagnostic::refused_fix(
                label.span,
                format!("my::{}", label.text),
                "a name of yours that clashes is written with `my::` (D75)",
            )));
        }
        let target = match label {
            None => body.loops.len().checked_sub(1),
            Some(label) => {
                let name = label.text.strip_prefix("my::").unwrap_or(&label.text);
                body.loops.iter().rposition(|each| each.label.as_deref() == Some(name))
            }
        };
        let Some(target) = target else {
            return Err(self.error(match label {
                None => Diagnostic::refused(
                    span,
                    "there's no loop around this for `break` to stop",
                    "`break;` stops the innermost loop (D128)",
                ),
                Some(label) => Diagnostic::refused(
                    span,
                    format!("there's no loop called `{}` around this", label.text),
                    "`break` with a loop's name stops it and every loop inside it (D128)",
                ),
            }));
        };
        let states = body.states.clone();
        let id = body.loops[target].id;
        if let Some(breaks) = &mut body.loops[target].breaks {
            breaks.push(states);
        }
        block.push(checked::Statement::Break(id));
        Ok(())
    }

    /// `for $i in range[…]:` (D128, D145, D146). Collections come later.
    fn for_statement(&mut self, body: &mut Body, each: &syntax::For, block: &mut checked::Block) -> Check<()> {
        let ExpressionKind::Bracket { base, labels, .. } = &each.source.kind else {
            return Err(self.not_yet(each.source.span, "`for` over this".to_string()));
        };
        match &base.kind {
            ExpressionKind::Path(path) if path.is_single("range") => {}
            _ => return Err(self.not_yet(each.source.span, "`for` over this".to_string())),
        }
        let range = self.range(body, each.source.span, labels)?;
        // The loop counts in 128 bits, so a step wider than the range's type, or one past its end, can't wrap
        // (D146).
        let wide = MachineType::Integer { bits: 128, signed: true };
        let widen = |value: checked::Expression| checked::Expression {
            kind: Kind::Convert(Box::new(value)),
            machine_type: wide.clone(),
        };
        let from = body.new_local(wide.clone());
        let end = body.new_local(wide.clone());
        let step = body.new_local(wide.clone());
        let at = body.new_local(wide.clone());
        let item_type = range.item_type.clone();
        let item = body.new_local(item_type.machine());
        let read =
            |local: checked::LocalId| checked::Expression { kind: Kind::Local(local), machine_type: wide.clone() };
        let number = |value: i128| checked::Expression { kind: Kind::Integer(value), machine_type: wide.clone() };
        block.push(checked::Statement::Let { local: from, value: widen(range.from) });
        block.push(checked::Statement::Let { local: end, value: widen(range.end) });
        block.push(checked::Statement::Let {
            local: step,
            value: match range.step {
                Some(step) => widen(step),
                None => number(1),
            },
        });
        if let Some(failure) = range.zero_step {
            block.push(checked::Statement::Check {
                condition: compare(checked::Comparison::NotEqual, read(step), number(0)),
                failure,
                operands: vec![read(from), read(end)],
            });
        }
        block.push(checked::Statement::Let { local: at, value: read(from) });
        // Going up, it runs while at most the end, or below it with `before:`, and going down the other way.
        let (up, down) = if range.before {
            (checked::Comparison::Below, checked::Comparison::Above)
        } else {
            (checked::Comparison::AtMost, checked::Comparison::AtLeast)
        };
        let going_up = compare(checked::Comparison::Above, read(step), number(0));
        let keeps_going = or(
            and(going_up.clone(), compare(up, read(at), read(end))),
            and(not(going_up), compare(down, read(at), read(end))),
        );
        body.scopes.push(Vec::new());
        let before = body.states.len();
        body.declare(
            Variable {
                name: each.name.text.clone(),
                value_type: item_type.clone(),
                mutable: false,
                origin: Origin::LoopItem,
                local: item,
                declared: each.name.span,
                given: None,
            },
            State::Assigned,
        );
        let start = body.states[..before].to_vec();
        let (id, loop_body, breaks) = self.loop_body(body, each.label.as_ref(), &each.body);
        body.scopes.pop();
        body.forget_after(before);
        let mut joined = Body::join_states(&start, &body.states[..before]);
        for state in breaks {
            joined = Body::join_states(&joined, &state[..before]);
        }
        body.states = joined;
        let mut statements = vec![
            checked::Statement::If {
                condition: not(keeps_going),
                then: vec![checked::Statement::Break(id)],
                otherwise: Vec::new(),
            },
            checked::Statement::Let {
                local: item,
                value: checked::Expression {
                    kind: Kind::Convert(Box::new(read(at))),
                    machine_type: item_type.machine(),
                },
            },
        ];
        statements.extend(loop_body);
        statements.push(checked::Statement::Set {
            local: at,
            value: checked::Expression {
                kind: Kind::Arithmetic {
                    operator: checked::Arithmetic::WrappingAdd,
                    left: Box::new(read(at)),
                    right: Box::new(read(step)),
                    failure: None,
                    more: Vec::new(),
                },
                machine_type: wide.clone(),
            },
        });
        block.push(checked::Statement::Loop { id, body: statements });
        Ok(())
    }

    /// Reads a range's labels (D128, D145, D146).
    fn range(&mut self, body: &mut Body, span: Span, labels: &[syntax::Label]) -> Check<Range> {
        let mut from = None;
        let mut end: Option<(&syntax::Label, bool)> = None;
        let mut step = None;
        let mut order = 0;
        for label in labels {
            let name = label.name.text();
            if label.sets {
                let between = Span::new(self.file, label.name.span.end as usize, label.value.span.start as usize);
                let offset = self.slice(between).find('=').unwrap_or(0);
                let place = Span::new(self.file, between.start as usize + offset, between.start as usize + offset + 1);
                return Err(self.error(Diagnostic::refused_fix(
                    place,
                    ":",
                    "a label names a value inside brackets (D13)",
                )));
            }
            let rank = match name.as_str() {
                "from" => 1,
                "to" | "before" => 2,
                "step_by" => 3,
                _ => {
                    return Err(self.error(Diagnostic::refused(
                        label.name.span,
                        format!("a range has no `{name}:`"),
                        RANGE_ORDER_WHY,
                    )));
                }
            };
            if rank <= order {
                return Err(self.error(Diagnostic::refused(
                    label.name.span,
                    format!("`{name}:` is out of order here"),
                    RANGE_ORDER_WHY,
                )));
            }
            order = rank;
            match name.as_str() {
                "from" => from = Some(label),
                "to" => end = Some((label, false)),
                "before" => end = Some((label, true)),
                _ => step = Some(label),
            }
        }
        let Some(from) = from else {
            return Err(self.error(Diagnostic::refused(span, "a range needs `from:`", RANGE_ORDER_WHY)));
        };
        let Some((end, before)) = end else {
            return Err(self.error(Diagnostic::refused(
                span,
                "this range has no end, so it can never reach one",
                "a range that can never reach its end is refused, and an endless loop is `forever` (D128)",
            )));
        };
        let (from_typed, end_typed) = self.range_ends(body, span, &from.value, &end.value)?;
        let item_type = from_typed.value_type.plain();
        let step_typed = match step {
            Some(step) => {
                let typed = self.expression(body, &step.value, Some(&item_type))?;
                self.range_number(&typed, &step.value)?;
                Some(typed)
            }
            None => None,
        };
        let never = || {
            Diagnostic::refused(
                span,
                "this range can never reach its end",
                "a range that can never reach its end is refused, and a negative `step_by:` counts down (D128)",
            )
        };
        let step_literal = step_typed.as_ref().map_or(Some(1), |typed| typed.literal);
        if step_literal == Some(0) {
            return Err(self.error(never()));
        }
        if let (Some(start), Some(stop), Some(by)) = (from_typed.literal, end_typed.literal, step_literal) {
            let reachable = match (by > 0, before) {
                (true, true) => start < stop,
                (true, false) => start <= stop,
                (false, true) => start > stop,
                (false, false) => start >= stop,
            };
            if !reachable {
                return Err(self.error(never()));
            }
        }
        let zero_step = match (&step_typed, step) {
            (Some(typed), Some(step)) if typed.literal.is_none() => {
                let ending = if before { "ending before" } else { "ending at" };
                Some(failure(
                    self,
                    Diagnostic::new(
                        DiagnosticKind::CheckFailed,
                        step.value.span,
                        "`step_by:` is 0, and nothing handled it",
                        "a step of 0 is a failure, since it never ends (D145, D96)",
                    ),
                    &[
                        When::Text("from ".into()),
                        When::Operand(0),
                        When::Text(format!(", {ending} ")),
                        When::Operand(1),
                    ],
                ))
            }
            _ => None,
        };
        Ok(Range {
            item_type,
            from: from_typed.value,
            end: end_typed.value,
            before,
            step: step_typed.map(|typed| typed.value),
            zero_step,
        })
    }

    /// Refuses a range value that isn't a whole number (D128).
    fn range_number(&mut self, typed: &Typed, value: &syntax::Expression) -> Check<()> {
        match &typed.value_type {
            Type::Integer { .. } => Ok(()),
            Type::Float { .. } => {
                Err(self.error(Diagnostic::not_designed(value.span, "a range of floats", NOT_DECIDED)))
            }
            Type::Unknown => Err(Reported),
            other => Err(self.error(Diagnostic::refused(
                value.span,
                format!("a range is a sequence of numbers, and this is `{other}`"),
                "a range is a sequence of numbers (D128)",
            ))),
        }
    }

    /// A range's start and end, which share one integer type (D128).
    fn range_ends(
        &mut self,
        body: &mut Body,
        span: Span,
        from: &syntax::Expression,
        end: &syntax::Expression,
    ) -> Check<(Typed, Typed)> {
        let from_result = self.expression(body, from, None);
        let end_result = self.expression(body, end, None);
        let (mut from_typed, mut end_typed) = (from_result?, end_result?);
        self.range_number(&from_typed, from)?;
        self.range_number(&end_typed, end)?;
        // A literal end takes the other's type.
        if from_typed.literal.is_some() && end_typed.literal.is_none() {
            from_typed = self.expression(body, from, Some(&end_typed.value_type.plain()))?;
        } else if end_typed.literal.is_some() && from_typed.literal.is_none() {
            end_typed = self.expression(body, end, Some(&from_typed.value_type.plain()))?;
        }
        if from_typed.value_type.plain() != end_typed.value_type.plain() {
            return Err(self.error(Diagnostic::refused(
                span,
                format!("this range mixes `{}` and `{}`", from_typed.value_type.plain(), end_typed.value_type.plain()),
                CONVERT_WHY,
            )));
        }
        Ok((from_typed, end_typed))
    }
}

/// A range, checked.
pub struct Range {
    pub item_type: Type,
    pub from: checked::Expression,
    pub end: checked::Expression,
    /// `before:` rather than `to:`.
    pub before: bool,
    pub step: Option<checked::Expression>,
    /// The failure when a step known only while running is 0 (D145).
    pub zero_step: Option<checked::CheckFailure>,
}

fn compare(
    operator: checked::Comparison,
    left: checked::Expression,
    right: checked::Expression,
) -> checked::Expression {
    checked::Expression {
        kind: Kind::Compare { operator, left: Box::new(left), right: Box::new(right) },
        machine_type: MachineType::Bool,
    }
}

fn not(value: checked::Expression) -> checked::Expression {
    checked::Expression { kind: Kind::Not(Box::new(value)), machine_type: MachineType::Bool }
}

fn and(left: checked::Expression, right: checked::Expression) -> checked::Expression {
    checked::Expression { kind: Kind::And(Box::new(left), Box::new(right)), machine_type: MachineType::Bool }
}

fn or(left: checked::Expression, right: checked::Expression) -> checked::Expression {
    checked::Expression { kind: Kind::Or(Box::new(left), Box::new(right)), machine_type: MachineType::Bool }
}
