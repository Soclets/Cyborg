//! Statements, and how ownership and trust flow through them.

use std::collections::HashMap;

use super::expression::{is_number_literal, literal_number};
use super::hand::UseKind;
use super::{
    ArmInfo, Checker, Failure, FunctionContext, LoopInfo, Origin, Variable, VariableState,
    ITEM_LIMIT,
};
use crate::ast::{
    Arm, Contract, Expression, ExpressionKind, ForSource, Function, FunctionKind, HandWord, IfLink,
    IfTest, ParameterWord, RangeExpression, Statement, StatementKind, TypeExpression, TypeKind,
};
use crate::ir::{self, Instruction, InstructionKind, Node, NodeKind, Place};
use crate::report::{Outcome, Report, ReportExtra, Span};
use crate::syntax::is_clashing;
use crate::types::{FloatRules, Sources, Type, Value};

/// A range's `from:`, its end, whether the end is included, and its `step_by:`.
type RangeNodes = (Node, Node, bool, Option<Node>);

/// A block's code, and whether it can't run past its end.
type CheckedBlock = Outcome<(Vec<Instruction>, bool)>;

/// What a `for` goes through: a value it reads, or a place whose items it changes. (D180)
enum LoopCollection {
    Value(Node),
    Place(Place),
}

/// What a `for` does to its collection while it runs. (D180, D182)
enum LoopSource {
    /// Bare: reading in place, so nothing may change it.
    Read(String, Vec<usize>),
    /// `Lend.mut`: changing it through each item, so nothing else may use it.
    Change(String),
    /// `Give`: taking each item, so it's gone, and what's left is owed until the end.
    Take { name: String, given: Span },
}

impl Checker<'_> {
    pub(super) fn check_function(&mut self, function: &Function) -> Outcome<ir::Function> {
        let signature = self
            .signatures
            .get(&function.name.text)
            .cloned()
            .ok_or_else(|| {
                Report::refused(function.name.span, "this function isn't declared", "(D23)")
            })?;
        let mut context = FunctionContext::outside_any_function();
        context.kind = function.kind;
        context.name.clone_from(&function.name.text);
        context.return_type = signature.return_type.clone();
        context.failures = function.fails.as_ref().map(|_| signature.failures.clone());
        self.context = context;

        // trace: D86
        if matches!(function.kind, FunctionKind::Pure | FunctionKind::PureBaked)
            && function.fails.is_some()
        {
            return Err(Report::not_designed(
                function.span,
                "whether a `fn.pure` may fail",
                "12.5",
            ));
        }

        for (parameter, info) in function.parameters.iter().zip(&signature.parameters) {
            if self
                .context
                .scopes
                .iter()
                .any(|scope| scope.contains_key(&info.name))
            {
                return Err(Report::refused(
                    parameter.name.span,
                    format!("`${}` is already a parameter here", info.name),
                    "a parameter's name is how the body reaches it (D2)",
                ));
            }
            let origin = match info.word {
                ParameterWord::Bare => Origin::LentParameter,
                ParameterWord::Given => Origin::GivenParameter,
                ParameterWord::LentMut => Origin::LentMutParameter,
            };
            let must_give =
                info.word == ParameterWord::Given && self.is_must_give(&info.parameter_type);
            self.declare_variable(Variable {
                name: info.name.clone(),
                held_type: info.parameter_type.clone(),
                mutable: info.word == ParameterWord::LentMut,
                origin,
                declared: parameter.name.span,
                must_give,
                constant: None,
                borrows: Vec::new(),
                borrowed_path: Vec::new(),
            });
        }

        // Contracts see the parameters as the function starts, and `ensures` sees `result` too.
        let result_slot = self.new_hidden_slot(&signature.return_type, function.span);
        self.context.result_slot = result_slot;
        let requires = self.check_contracts(&function.requires, false)?;
        let ensures = self.check_contracts(&function.ensures, true)?;
        for contract in &function.ensures {
            collect_variables(&contract.condition, &mut self.context.ensures_variables);
        }

        // trace: D114
        for (parameter, value) in &function.defaults {
            if let Some(info) = signature
                .parameters
                .iter()
                .find(|info| info.name == parameter.text)
            {
                self.check_argument(value, info, &mut Vec::new())?;
            }
        }
        let (body, terminated) = self.check_block(&function.body)?;
        Self::check_function_end(function, terminated, &signature.return_type)?;
        self.check_scope_end_at_function(function.end_span)?;
        Ok(ir::Function {
            slot_count: self.context.variables.len(),
            result_slot,
            requires,
            ensures,
            body,
        })
    }

    fn check_contracts(
        &mut self,
        contracts: &[Contract],
        ensures: bool,
    ) -> Outcome<Vec<ir::Contract>> {
        let mut checked = Vec::new();
        for contract in contracts {
            self.context.in_contract = true;
            self.context.in_ensures = ensures;
            let result = self.check_condition(&contract.condition);
            self.context.in_contract = false;
            self.context.in_ensures = false;
            let condition = result?;
            let mut variables = Vec::new();
            collect_variables(&contract.condition, &mut variables);
            let mut shown = Vec::new();
            for (name, span) in variables {
                if let Some(slot) = self.find_variable(&name) {
                    let is_simple = self.context.variables.get(slot).is_some_and(|variable| {
                        matches!(
                            variable.held_type,
                            Type::Integer { .. } | Type::Bool | Type::Float { .. }
                        )
                    });
                    if is_simple {
                        shown.push((
                            format!("${name}"),
                            Node {
                                span,
                                kind: NodeKind::Read(Place {
                                    slot,
                                    path: Vec::new(),
                                }),
                            },
                        ));
                    }
                }
            }
            if ensures && mentions_result(&contract.condition) {
                shown.push((
                    "result".to_string(),
                    Node {
                        span: contract.condition.span,
                        kind: NodeKind::Read(Place {
                            slot: self.context.result_slot,
                            path: Vec::new(),
                        }),
                    },
                ));
            }
            checked.push(ir::Contract { condition, shown });
        }
        Ok(checked)
    }

    fn check_function_end(
        function: &Function,
        terminated: bool,
        return_type: &Type,
    ) -> Outcome<()> {
        if terminated {
            return Ok(());
        }
        if function.kind == FunctionKind::Main {
            // trace: D11
            return Err(Report::refused(
                function.end_span,
                "`fn.main` ends without `return void;`",
                "`main` says `return void;` (D11)",
            ));
        }
        if *return_type == Type::Void {
            return Err(Report::not_designed(
                function.end_span,
                "whether every `void` function says `return void;`",
                "12.5",
            ));
        }
        Err(Report::refused(
            function.end_span,
            format!(
                "`{}` can reach its end without a `return`",
                function.name.text
            ),
            "`return` hands back a value (D11)",
        ))
    }

    fn new_hidden_slot(&mut self, value_type: &Type, span: Span) -> usize {
        self.context.variables.push(Variable {
            name: String::new(),
            held_type: value_type.clone(),
            mutable: false,
            origin: Origin::Owned,
            declared: span,
            must_give: false,
            constant: None,
            borrows: Vec::new(),
            borrowed_path: Vec::new(),
        });
        self.context
            .state
            .resize(self.context.variables.len(), VariableState::default());
        self.context.variables.len() - 1
    }

    pub(super) fn declare_variable(&mut self, variable: Variable) -> usize {
        let slot = self.context.variables.len();
        let name = variable.name.clone();
        self.context.variables.push(variable);
        // A finished branch hands back a shorter state than the variables it declared, so this
        // fills the gap: a variable's state is always found by its slot.
        self.context
            .state
            .resize(slot + 1, VariableState::default());
        if let Some(scope) = self.context.scopes.last_mut() {
            scope.insert(name, slot);
        }
        slot
    }

    pub(super) fn find_variable(&self, name: &str) -> Option<usize> {
        self.context
            .scopes
            .iter()
            .rev()
            .find_map(|scope| scope.get(name).copied())
    }

    /// A name that's already taken here: shadowing isn't in the standard, so it isn't guessed.
    fn refuse_second_declaration(&self, name: &crate::ast::Name) -> Outcome<()> {
        if self.find_variable(&name.text).is_some() || self.globals.contains_key(&name.text) {
            return Err(Report::not_designed(
                name.span,
                format!(
                    "declaring `${}` again while the first is in scope",
                    name.text
                ),
                "",
            ));
        }
        Ok(())
    }

    /// Checks a block in its own scope, and says whether it can't run past its end.
    pub(super) fn check_block(
        &mut self,
        statements: &[Statement],
    ) -> Outcome<(Vec<Instruction>, bool)> {
        self.context.scopes.push(HashMap::new());
        let result = self.check_statements(statements);
        let end_span = statements.last().map_or(Span::new(0, 0), |last| last.span);
        let (code, terminated) = match result {
            Ok(done) => done,
            Err(report) => {
                self.context.scopes.pop();
                return Err(report);
            }
        };
        self.end_held_borrows();
        let scope_check = if terminated {
            Ok(())
        } else {
            self.check_innermost_scope_end(end_span)
        };
        self.context.scopes.pop();
        scope_check?;
        Ok((code, terminated))
    }

    fn check_statements(&mut self, statements: &[Statement]) -> Outcome<(Vec<Instruction>, bool)> {
        let mut code = Vec::new();
        let mut terminated = false;
        for statement in statements {
            let (instruction, ends) = self.check_statement(statement)?;
            self.refuse_use_while_held()?;
            code.push(instruction);
            terminated = terminated || ends;
        }
        Ok((code, terminated))
    }

    /// A stored borrow ends at its last use, so one whose source was used and which is never read
    /// again was fine. A `.held` one lasts its whole block, so a use that would end it is
    /// refused there and then. (D169)
    fn refuse_use_while_held(&mut self) -> Outcome<()> {
        let Some((used_at, slot)) = self.context.held_conflict.take() else {
            return Ok(());
        };
        let Some(variable) = self.context.variables.get(slot) else {
            return Ok(());
        };
        let source = variable
            .borrows
            .first()
            .map_or(String::new(), |name| format!("${name}"));
        Err(Report::refused(
            used_at,
            format!(
                "`{}` holds `{source}` until the end of its block",
                variable.name_shown()
            ),
            "`.held` means nobody touches what it borrows for the whole block (D169)",
        )
        .because(
            variable.declared,
            format!("`{}` holds it from here", variable.name_shown()),
        ))
    }

    fn end_held_borrows(&mut self) {
        let ended: Vec<usize> = self
            .context
            .scopes
            .last()
            .map(|scope| scope.values().copied().collect())
            .unwrap_or_default();
        self.context
            .held_borrows
            .retain(|slot| !ended.contains(slot));
    }

    /// trace: D2
    fn check_innermost_scope_end(&self, _span: Span) -> Outcome<()> {
        if let Some(scope) = self.context.scopes.last() {
            let mut slots: Vec<usize> = scope.values().copied().collect();
            slots.sort_unstable();
            for slot in slots {
                self.check_duty_ended(slot, None)?;
            }
        }
        Ok(())
    }

    fn check_scope_end_at_function(&self, _end: Span) -> Outcome<()> {
        if let Some(scope) = self.context.scopes.first() {
            let mut slots: Vec<usize> = scope.values().copied().collect();
            slots.sort_unstable();
            for slot in slots {
                self.check_duty_ended(slot, None)?;
            }
        }
        Ok(())
    }

    /// A `must_Give` value has to have been given or finished by the time its holder goes away.
    /// trace: D2, D129
    pub(super) fn check_duty_ended(&self, slot: usize, exit: Option<Span>) -> Outcome<()> {
        let Some(variable) = self.context.variables.get(slot) else {
            return Ok(());
        };
        if !variable.must_give {
            return Ok(());
        }
        let given = self.context.state.get(slot).and_then(|state| state.given);
        let only_some_path = self
            .context
            .state
            .get(slot)
            .is_some_and(|state| state.given_on_some_path);
        if given.is_some() && !only_some_path {
            return Ok(());
        }
        let why = "a `must_Give` value is handed off with `Give` exactly once, or `finish` ends it (D2, D129)";
        if let Some(given_at) = given {
            return Err(Report::refused(
                variable.declared,
                format!(
                    "`${}` isn't given away or finished on every path",
                    variable.name
                ),
                why,
            )
            .because(
                given_at,
                format!("`${}` went here, on only one path", variable.name),
            ));
        }
        // trace: D180
        if variable.origin == Origin::LoopRest {
            let mut report = Report::refused(
                exit.unwrap_or(variable.declared),
                format!(
                    "this leaves before the `for` has handed out every item of `${}`",
                    variable.name
                ),
                "a `must_Give` value is handed off with `Give` exactly once, so every item has to be (D2, D180)",
            );
            if exit.is_some() {
                report = report.because(
                    variable.declared,
                    format!("`${}` was given to the `for` here", variable.name),
                );
            }
            return Err(report);
        }
        let report = match exit {
            Some(exit_span) => Report::refused(
                exit_span,
                format!(
                    "this leaves with `${}` never given away or finished",
                    variable.name
                ),
                why,
            )
            .because(
                variable.declared,
                if variable.origin == Origin::GivenParameter {
                    format!("`${}` was given to this function here", variable.name)
                } else {
                    format!("`${}` was made here", variable.name)
                },
            ),
            None => Report::refused(
                variable.declared,
                format!("`${}` is never given away or finished", variable.name),
                why,
            ),
        };
        Err(report)
    }

    /// `finish $f;` ends a `must_Give` duty. Only the kit or file that declared the type can write
    /// it, and with no modules yet, a program is one file, so the type is always its own.
    /// trace: D129
    fn check_finish(&mut self, target: &Expression, span: Span) -> Outcome<InstructionKind> {
        const WHY: &str = "`finish` ends a `must_Give` duty (D129)";
        let ExpressionKind::Variable(name) = &target.kind else {
            if matches!(
                target.kind,
                ExpressionKind::Field { .. } | ExpressionKind::Item { .. }
            ) {
                return Err(Report::not_designed(
                    target.span,
                    "`finish` on a field, since a `must_Give` value in a field isn't decided",
                    "12.8",
                ));
            }
            return Err(Report::refused(
                target.span,
                "`finish` goes on a variable that holds the duty",
                WHY,
            ));
        };
        let Some(slot) = self.find_variable(name) else {
            return Err(Report::refused(
                target.span,
                format!("`${name}` isn't declared"),
                WHY,
            ));
        };
        let finished_at = self
            .context
            .state
            .get(slot)
            .filter(|state| state.finished)
            .and_then(|state| state.given);
        if let Some(finished_at) = finished_at {
            return Err(Report::refused(
                span,
                format!("`${name}` was already finished"),
                "a `must_Give` duty ends once (D2, D129)",
            )
            .because(finished_at, format!("`${name}` was finished here")));
        }
        self.require_alive(slot, span)?;
        let Some(variable) = self.context.variables.get(slot).cloned() else {
            return Ok(InstructionKind::Block(Vec::new()));
        };
        if !self.is_must_give(&variable.held_type) {
            return Err(Report::refused(
                target.span,
                format!(
                    "`${name}` is `{}`, which has no duty to end",
                    variable.held_type
                ),
                WHY,
            ));
        }
        if !matches!(variable.origin, Origin::Owned | Origin::GivenParameter) {
            return Err(Report::refused(
                target.span,
                format!("`${name}` is only lent here, so its duty isn't this code's to end"),
                "`Lend` is temporary access, and the caller keeps it (D2, D129)",
            ));
        }
        self.refuse_finish_with_duties_inside(name, &variable.held_type, target.span)?;
        self.refuse_while_read(name, &[], target.span)?;
        self.note_use(name, UseKind::Write, target.span);
        if let Some(state) = self.context.state.get_mut(slot) {
            state.given = Some(span);
            state.given_on_some_path = false;
            state.finished = true;
        }
        Ok(InstructionKind::Block(Vec::new()))
    }

    /// `finish` ends a value's own duty, and one inside it still has to be given away first.
    /// trace: D129, D177
    fn refuse_finish_with_duties_inside(
        &self,
        name: &str,
        held_type: &Type,
        span: Span,
    ) -> Outcome<()> {
        match held_type {
            Type::Struct(struct_name) => {
                let fields = self
                    .structs
                    .get(struct_name)
                    .map(|info| info.fields.clone())
                    .unwrap_or_default();
                if let Some((field, _)) = fields
                    .iter()
                    .find(|(_, field_type)| self.is_must_give(field_type))
                {
                    // Giving one field away isn't decided yet, so it can't have happened.
                    return Err(Report::refused(
                        span,
                        format!("`${name}.{field}` hasn't been given away, so `finish` waits for it"),
                        "`finish` ends a value's own duty, after every `must_Give` value inside it is given away (D129, D177)",
                    ));
                }
                Ok(())
            }
            Type::List { .. } | Type::Array { .. } => Err(Report::not_designed(
                span,
                "`finish` on a collection of `must_Give` values",
                "",
            )),
            _ => Ok(()),
        }
    }

    /// Leaving the function early: every duty still open is forgotten on this path. (D2)
    pub(super) fn check_duties_at_exit(&self, exit: Span) -> Outcome<()> {
        for scope in &self.context.scopes {
            let mut slots: Vec<usize> = scope.values().copied().collect();
            slots.sort_unstable();
            for slot in slots {
                self.check_duty_ended(slot, Some(exit))?;
            }
        }
        Ok(())
    }

    fn join_states(&mut self, branches: &[Vec<VariableState>]) {
        let count = self.context.state.len();
        let mut joined: Vec<VariableState> = self.context.state.clone();
        let branch_count = branches.len();
        for (index, slot_state) in joined.iter_mut().enumerate().take(count) {
            let mut given_count = 0;
            let mut given = None;
            let mut trusts = Vec::new();
            let mut trust_ended = None;
            let mut invalidated = None;
            let mut finished = false;
            let mut matched = false;
            let mut some_path = false;
            for branch in branches {
                let Some(state) = branch.get(index) else {
                    continue;
                };
                if let Some(span) = state.given {
                    given_count += 1;
                    given = given.or(Some(span));
                }
                trusts.push(state.trust);
                trust_ended = trust_ended.or(state.trust_ended);
                invalidated = invalidated.or(state.invalidated);
                finished = finished || state.finished;
                matched = matched || state.matched;
                some_path = some_path || state.given_on_some_path;
            }
            slot_state.given = given;
            slot_state.given_on_some_path =
                some_path || (given_count > 0 && given_count < branch_count);
            slot_state.invalidated = invalidated;
            slot_state.finished = finished;
            slot_state.matched = matched;
            let first = trusts.first().copied().flatten();
            slot_state.trust = if trusts.iter().all(|trust| *trust == first) {
                first
            } else {
                None
            };
            slot_state.trust_ended = trust_ended;
        }
        self.context.state = joined;
    }

    /// Checks something that runs on some paths and not others, so that what it does to ownership
    /// only counts as maybe having happened.
    pub(super) fn on_one_path_only<T>(
        &mut self,
        check: impl FnOnce(&mut Self) -> Outcome<T>,
    ) -> Outcome<T> {
        let before = self.context.state.clone();
        let (value, (), branch) = self.on_a_branch(|this| Ok((check(this)?, ())))?;
        self.join_states(&[before, branch]);
        Ok(value)
    }

    /// Runs `check` on a copy of the state, and returns what it did and the state it left.
    fn on_a_branch<T, U>(
        &mut self,
        check: impl FnOnce(&mut Self) -> Outcome<(T, U)>,
    ) -> Outcome<(T, U, Vec<VariableState>)> {
        let saved = self.context.state.clone();
        let (value, terminated) = check(self)?;
        let branch = std::mem::replace(&mut self.context.state, saved);
        Ok((value, terminated, branch))
    }

    /// Going round a loop again: what the first time through did is still true the second time.
    fn check_loop_body(
        &mut self,
        check: &mut dyn FnMut(&mut Self) -> CheckedBlock,
    ) -> Outcome<Vec<Instruction>> {
        let before = self.context.state.clone();
        let (code, terminated, after_once) = self.on_a_branch(|this| check(this))?;
        if !terminated {
            self.join_states(&[before.clone(), after_once]);
            let (_, _, after_twice) = self.on_a_branch(|this| check(this))?;
            let current = self.context.state.clone();
            self.join_states(&[current, after_twice]);
        }
        Ok(code)
    }

    #[allow(clippy::too_many_lines)] // One arm per statement, read top to bottom.
    fn check_statement(&mut self, statement: &Statement) -> Outcome<(Instruction, bool)> {
        let span = statement.span;
        let mut terminated = false;
        let kind = match &statement.kind {
            StatementKind::Let {
                mutable,
                name,
                written_type,
                value,
            } => self.check_let(*mutable, name, written_type, value)?,
            StatementKind::Constant {
                name,
                written_type,
                value,
            } => self.check_local_constant(name, written_type, value)?,
            StatementKind::Change { target, value } => self.check_change(target, value, span)?,
            StatementKind::If { links, last } => {
                let (kind, ends) = self.check_if(links, last.as_ref())?;
                terminated = ends;
                kind
            }
            StatementKind::Match { subject, arms } => {
                let (kind, ends) = self.check_match(subject, arms, span)?;
                terminated = ends;
                kind
            }
            StatementKind::For {
                loop_name,
                name,
                source,
                body,
            } => self.check_for(loop_name.as_ref(), name, source, body)?,
            StatementKind::While {
                condition, body, ..
            } => self.check_while(condition, body)?,
            StatementKind::Forever { body, .. } => {
                let (kind, ends) = self.check_forever(body)?;
                terminated = ends;
                kind
            }
            StatementKind::Finish { target } => self.check_finish(target, span)?,
            StatementKind::Instead(value) => {
                terminated = true;
                self.check_instead(value, span)?
            }
            StatementKind::Break { name } => {
                terminated = true;
                self.check_break(name.as_ref(), span)?
            }
            StatementKind::Return(value) => {
                terminated = true;
                self.check_return(value, span)?
            }
            StatementKind::Fail { path, data } => {
                terminated = true;
                self.check_fail(path, data, span)?
            }
            StatementKind::Expression(value) => self.check_expression_statement(value)?,
            StatementKind::Shield { name, reason, body } => {
                // trace: D34
                if reason.is_none() {
                    return Err(Report::refused(
                        name.span,
                        format!("the shield `{}` has no reason", name.text),
                        "a shield is a named block with a written reason (D34)",
                    ));
                }
                // trace: D86
                if matches!(
                    self.context.kind,
                    FunctionKind::Pure | FunctionKind::PureBaked
                ) {
                    return Err(Report::refused(
                        name.span,
                        "a pure function can't hold a shield",
                        "a pure function refuses shields and C calls (D86)",
                    ));
                }
                self.context.in_shield += 1;
                let result = self.check_block(body);
                self.context.in_shield -= 1;
                let (code, ends) = result?;
                terminated = ends;
                InstructionKind::Block(code)
            }
        };
        Ok((Instruction { span, kind }, terminated))
    }

    fn check_let(
        &mut self,
        mutable: bool,
        name: &crate::ast::Name,
        written_type: &TypeExpression,
        value: &Expression,
    ) -> Outcome<InstructionKind> {
        self.refuse_second_declaration(name)?;
        // `held` only says how long a `let` keeps its borrow, so the type is read without it.
        // trace: D169
        if let TypeKind::StoredBorrow {
            mutable: lends_mut,
            held,
            sources,
            inner,
        } = &written_type.kind
        {
            let plain = TypeExpression {
                span: written_type.span,
                kind: TypeKind::StoredBorrow {
                    mutable: *lends_mut,
                    held: false,
                    sources: sources.clone(),
                    inner: inner.clone(),
                },
            };
            let held_type = self.resolve_type(&plain)?;
            return self.check_stored_borrow(mutable, *held, name, &held_type, value);
        }
        let held_type = self.resolve_type(written_type)?;
        // trace: D2
        let must_give = self.is_must_give(&held_type);
        let node = self.check_into(value, &held_type)?;
        let slot = self.declare_variable(Variable {
            name: name.text.clone(),
            held_type,
            mutable,
            origin: Origin::Owned,
            declared: name.span,
            must_give,
            constant: None,
            borrows: Vec::new(),
            borrowed_path: Vec::new(),
        });
        Ok(InstructionKind::Let { slot, value: node })
    }

    fn check_local_constant(
        &mut self,
        name: &crate::ast::Name,
        written_type: &TypeExpression,
        value: &Expression,
    ) -> Outcome<InstructionKind> {
        self.refuse_second_declaration(name)?;
        let held_type = self.resolve_type(written_type)?;
        let node = self.check_constant_value(value, &held_type)?;
        self.compile_time.push(node.clone());
        let slot = self.declare_variable(Variable {
            name: name.text.clone(),
            held_type,
            mutable: false,
            origin: Origin::Constant,
            declared: name.span,
            must_give: false,
            constant: Some(node.clone()),
            borrows: Vec::new(),
            borrowed_path: Vec::new(),
        });
        Ok(InstructionKind::Let { slot, value: node })
    }

    /// What a stored borrow really borrows, which its type has to name. (D2, D174)
    fn borrow_roots_named(&self, root: &str, names: &[String], span: Span) -> Outcome<Vec<String>> {
        let roots = self.borrow_roots(root);
        let through = roots != [root.to_string()];
        if through && !roots.iter().all(|root| names.contains(root)) {
            let shown = roots
                .iter()
                .map(|n| format!("${n}"))
                .collect::<Vec<_>>()
                .join(", ");
            return Err(Report::refused(
                span,
                format!("this borrows `{shown}` through `${root}`, so the type names `{shown}`"),
                "a borrow taken through a stored borrow names the original value (D174)",
            ));
        }
        if roots.is_empty() || !roots.iter().all(|root| names.contains(root)) {
            return Err(Report::refused(
                span,
                format!(
                    "this borrow says it's from `{}`",
                    names
                        .iter()
                        .map(|n| format!("${n}"))
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
                "a stored borrow's type says what it borrows (D2)",
            ));
        }
        Ok(roots)
    }

    /// trace: D2
    fn check_stored_borrow(
        &mut self,
        mutable: bool,
        held: bool,
        name: &crate::ast::Name,
        held_type: &Type,
        value: &Expression,
    ) -> Outcome<InstructionKind> {
        let Type::Borrow {
            mutable: borrow_mutable,
            sources,
            inner,
        } = held_type
        else {
            return Err(Report::refused(
                name.span,
                "this isn't a stored borrow",
                "(D2)",
            ));
        };
        if mutable {
            return Err(Report::not_designed(
                name.span,
                "`let mut` on a stored borrow",
                "12.8",
            ));
        }
        let Sources::Variables(names) = sources else {
            return Err(Report::not_designed(
                value.span,
                "what can be lent `from program`",
                "12.8",
            ));
        };
        let ExpressionKind::Hand { word, value: lent } = &value.kind else {
            return Err(Report::refused(
                value.span,
                "a stored borrow is made with `Lend` or `Lend.mut`",
                "every hand-off shows on the page (D2, D3)",
            ));
        };
        let wanted = if *borrow_mutable {
            crate::ast::HandWord::LendMut
        } else {
            crate::ast::HandWord::Lend
        };
        if *word != wanted {
            return Err(Report::refused(
                value.span,
                format!("this borrow is made with `{}`", wanted.word()),
                "the call and the parameter both say it, and they must match (D2)",
            ));
        }
        let place = self.check_lent_place(lent, *word)?;
        // A borrow taken through a stored borrow names the original value. (D174)
        let roots = self.borrow_roots_named(&place.root_name, names, value.span)?;
        if place.value_type != **inner {
            return Err(Self::mismatch(value.span, inner, &place.value_type));
        }
        let Some(source) = place.place else {
            return Err(Report::not_designed(
                value.span,
                "a stored borrow of a constant",
                "12.8",
            ));
        };
        // Lending an item locks the whole list, since adding to it can move every item. (D172)
        let mut borrowed_path = source.keys();
        if let Some(first_item) = borrowed_path.iter().position(|key| *key == ir::ANY_ITEM) {
            borrowed_path.truncate(first_item);
        }
        // Through a stored borrow, the path is into the original, which isn't tracked this
        // finely, so it borrows the whole of it.
        let mut borrows = roots.clone();
        if !roots.contains(&place.root_name) {
            borrows.push(place.root_name.clone());
            borrowed_path = Vec::new();
        }
        let slot = self.declare_variable(Variable {
            name: name.text.clone(),
            held_type: held_type.clone(),
            mutable: *borrow_mutable,
            origin: Origin::StoredBorrow,
            declared: name.span,
            must_give: false,
            constant: None,
            borrows,
            borrowed_path,
        });
        if held {
            self.context.held_borrows.push(slot);
        }
        Ok(InstructionKind::LetAlias {
            slot,
            place: source,
        })
    }

    fn check_change(
        &mut self,
        target: &Expression,
        value: &Expression,
        span: Span,
    ) -> Outcome<InstructionKind> {
        let ExpressionKind::Variable(name) = &target.kind else {
            if matches!(
                target.kind,
                ExpressionKind::Field { .. } | ExpressionKind::Item { .. }
            ) {
                return self.check_change_field(target, value);
            }
            return Err(Report::refused(
                target.span,
                "`change` reassigns a variable",
                "`let mut` declares a variable that can change, and `change` reassigns it (D3)",
            ));
        };
        // trace: D83
        if self.find_variable(name).is_none() && self.globals.contains_key(name) {
            return Err(Report::refused(
                target.span,
                format!("`${name}` is a constant, so it can't change"),
                "a constant's value is fixed at compile time (D83)",
            ));
        }
        let slot = self.find_variable(name).ok_or_else(|| {
            Report::refused(
                target.span,
                format!("`${name}` isn't declared"),
                "`let` declares a variable (D2)",
            )
        })?;
        let variable = self.context.variables.get(slot).cloned().ok_or_else(|| {
            Report::refused(target.span, format!("`${name}` isn't declared"), "(D2)")
        })?;
        self.require_alive(slot, target.span)?;
        // trace: D3
        if !variable.mutable {
            return Err(Report::refused(
                target.span,
                format!("`${name}` can't change"),
                "`let` declares a variable that can't change, and `let mut` one that can (D3)",
            ));
        }
        self.refuse_while_read(name, &[], target.span)?;
        self.note_use(name, UseKind::Write, target.span);
        // trace: D2
        if variable.must_give {
            let still_alive = self
                .context
                .state
                .get(slot)
                .is_some_and(|state| state.given.is_none());
            if still_alive {
                return Err(Report::refused(
                    target.span,
                    format!("this `change` throws away `${name}` without giving it"),
                    "a `must_Give` value is handed off with `Give` exactly once (D2)",
                ));
            }
        }
        // trace: D3
        if matches!(value.kind, ExpressionKind::Method { percent: false, .. }) {
            return Err(Report::refused(
                value.span,
                "`change` doesn't stand in for an ownership word, and a `.` never runs code",
                "every hand-off shows on the page, including one to a method's receiver (D3, D8)",
            ));
        }
        let target_type = match &variable.held_type {
            Type::Borrow { inner, .. } => (**inner).clone(),
            other => other.clone(),
        };
        let node = self.check_into(value, &target_type)?;
        // trace: D104
        if let Some(state) = self.context.state.get_mut(slot) {
            if state.trust.is_some() {
                state.trust = None;
                state.trust_ended = Some(span);
            }
        }
        Ok(InstructionKind::Set {
            place: Place {
                slot,
                path: Vec::new(),
            },
            value: node,
        })
    }

    /// `change $r.celsius = 22;`: a field can change if the whole variable can. (D125)
    /// trace: D125
    fn check_change_field(
        &mut self,
        target: &Expression,
        value: &Expression,
    ) -> Outcome<InstructionKind> {
        if root_variable(target).is_none() {
            return Err(Report::refused(
                target.span,
                "`change` works on a variable, or a field of one",
                "`change` reassigns a variable or one field of it (D3, D125)",
            ));
        }
        let held = self.check_held(target)?;
        let Some(place) = held.place else {
            return Err(Report::refused(
                target.span,
                format!("`${}` is a constant, so it can't change", held.root_name),
                "a constant's value is fixed at compile time (D83)",
            ));
        };
        let can_change = self
            .context
            .variables
            .get(place.slot)
            .is_some_and(|variable| variable.mutable);
        if !can_change {
            let name = &held.root_name;
            let part = if matches!(target.kind, ExpressionKind::Item { .. }) {
                "item"
            } else {
                "field"
            };
            return Err(Report::refused(
                target.span,
                format!("`${name}` can't change, so neither can its {part}"),
                "a field or item can be changed if the whole variable is `let mut`, or the parameter `Lent.mut` (D125, D133)",
            ));
        }
        // A case's values aren't struct fields, and whether the test's trust survives isn't stated.
        let mut base = target;
        while let ExpressionKind::Field { base: inner, .. }
        | ExpressionKind::Item { base: inner, .. } = &base.kind
        {
            base = inner;
            if matches!(self.check_held(base)?.value_type, Type::Union(_)) {
                return Err(Report::not_designed(
                    target.span,
                    "`change` on a value inside a union's case, and whether the test's trust survives it",
                    "",
                ));
            }
        }
        // An item's place is worked out before the value, so the value can't be what moves it.
        let mut value_lends_it = false;
        walk(value, &mut |part| {
            let lent = match &part.kind {
                ExpressionKind::Hand {
                    word: HandWord::Give | HandWord::LendMut,
                    value: inner,
                } => root_variable(inner),
                ExpressionKind::Method { receiver, .. } => root_variable(receiver),
                _ => None,
            };
            value_lends_it = value_lends_it || lent.as_deref() == Some(held.root_name.as_str());
        });
        if value_lends_it {
            return Err(Report::not_designed(
                value.span,
                format!(
                    "changing part of `${}` with a value that lends it out",
                    held.root_name
                ),
                "",
            ));
        }
        let keys = place.keys();
        self.refuse_while_read(&held.root_name, &keys, target.span)?;
        // trace: D173
        self.note_use_at(&held.root_name, &keys, UseKind::Write, target.span);
        let node = self.check_into(value, &held.value_type)?;
        Ok(InstructionKind::Set { place, value: node })
    }

    /// `if`, each `else if`, and `else:`. Each test runs only if those before it were false, each
    /// block starts from what its own test left, and what comes after joins every way out. (D126)
    /// trace: D126
    fn check_if(
        &mut self,
        links: &[IfLink],
        last: Option<&Vec<Statement>>,
    ) -> Outcome<(InstructionKind, bool)> {
        let mut exits: Vec<Vec<VariableState>> = Vec::new();
        let mut code = Vec::new();
        let mut tested_slots = Vec::new();
        for link in links {
            let (test, trust) = match &link.test {
                IfTest::Condition(condition) => {
                    (ir::Test::Condition(self.check_condition(condition)?), None)
                }
                IfTest::Type {
                    subject,
                    written_type,
                } => self.check_test(subject, written_type)?,
            };
            let (body, ends, branch) = self.on_a_branch(|this| {
                if let Some((slot, case)) = trust {
                    if let Some(state) = this.context.state.get_mut(slot) {
                        state.trust = Some(case);
                        state.trust_ended = None;
                    }
                }
                this.check_block(&link.body)
            })?;
            if let Some((slot, _)) = trust {
                tested_slots.push(slot);
            }
            if !ends {
                exits.push(branch);
            }
            code.push((test, body));
        }
        let otherwise = if let Some(block) = last {
            let (body, ends, branch) = self.on_a_branch(|this| this.check_block(block))?;
            if !ends {
                exits.push(branch);
            }
            body
        } else {
            exits.push(self.context.state.clone());
            Vec::new()
        };
        let all_end = exits.is_empty();
        if !all_end {
            self.join_states(&exits);
        }
        // The trust ends with its block, so a read after this is the plain "outside" error. (D9)
        for slot in tested_slots {
            if let Some(state) = self.context.state.get_mut(slot) {
                state.trust = None;
                state.trust_ended = None;
            }
        }
        Ok((
            InstructionKind::If {
                links: code,
                otherwise,
            },
            all_end,
        ))
    }

    /// `$s: status.failed` and `$n: i32[min: 0, max: 100]`, without their blocks. It also says what
    /// the block may trust, as a variable's slot and a case. (D9, D20)
    fn check_test(
        &mut self,
        subject: &Expression,
        written_type: &TypeExpression,
    ) -> Outcome<(ir::Test, Option<(usize, usize)>)> {
        let ExpressionKind::Variable(name) = &subject.kind else {
            return Err(Report::not_designed(
                subject.span,
                "the test on anything but a variable",
                "12.4",
            ));
        };
        let (node, subject_type) = self.check_expression(subject, None)?;
        let slot = self.find_variable(name);
        // trace: D9
        if let Some((union_name, case_name)) = self.split_case_path(written_type) {
            let Type::Union(subject_union) = &subject_type else {
                return Err(Report::refused(
                    written_type.span,
                    format!("`${name}` is `{subject_type}`, so it has no such case"),
                    "`if $s: status.failed:` tests one case of a union (D9)",
                ));
            };
            if *subject_union != union_name {
                return Err(Report::refused(
                    written_type.span,
                    format!("`${name}` is `{subject_union}`, not `{union_name}`"),
                    "`if $s: status.failed:` tests one case of a union (D9)",
                ));
            }
            let case = self.find_case(&union_name, &case_name).ok_or_else(|| {
                Report::refused(
                    written_type.span,
                    format!("`{union_name}` has no case `{case_name}`"),
                    "a union's cases are narrowings (D33)",
                )
            })?;
            let already_trusted = slot
                .and_then(|slot| self.context.state.get(slot))
                .is_some_and(|state| state.trust.is_some());
            if already_trusted {
                return Err(Report::not_designed(
                    written_type.span,
                    "the test on a value already known to be that type",
                    "12.2",
                ));
            }
            return Ok((
                ir::Test::Case {
                    subject: node,
                    case,
                },
                slot.map(|slot| (slot, case)),
            ));
        }
        let test_type = self.resolve_type(written_type)?;
        // trace: D20
        if let (
            Type::Integer {
                kind,
                bound: None,
                unit,
            },
            Type::Integer {
                kind: test_kind,
                bound: Some((low, high)),
                unit: test_unit,
            },
        ) = (&subject_type, &test_type)
        {
            if kind == test_kind && unit == test_unit {
                return Ok((
                    ir::Test::Range {
                        subject: node,
                        low: *low,
                        high: *high,
                    },
                    None,
                ));
            }
        }
        if let Some((low, high)) = float_test_bound(&subject_type, &test_type) {
            return Ok((
                ir::Test::FloatRange {
                    subject: node,
                    low,
                    high,
                },
                None,
            ));
        }
        Err(Report::not_designed(
            written_type.span,
            format!("the test for `{test_type}` on `{subject_type}`"),
            "12.2",
        ))
    }

    /// Reads a test's type as `union.case`, if it's written that way.
    fn split_case_path(&self, written_type: &TypeExpression) -> Option<(String, String)> {
        let TypeKind::Named {
            path,
            fill,
            describe,
        } = &written_type.kind
        else {
            return None;
        };
        if !fill.is_empty() || !describe.is_empty() {
            return None;
        }
        let text = self.through_aliases(&self.canonical(&path.text));
        let text = text.strip_prefix("my::").unwrap_or(&text);
        let (union_name, case_name) = text.split_once('.')?;
        self.unions
            .contains_key(union_name)
            .then(|| (union_name.to_string(), case_name.to_string()))
    }

    pub(super) fn find_case(&self, union_name: &str, case_name: &str) -> Option<usize> {
        self.unions
            .get(union_name)?
            .cases
            .iter()
            .position(|(case, _)| case == case_name)
    }

    /// trace: D9
    #[allow(clippy::too_many_lines)] // Every rule a `match` has, top to bottom.
    fn check_match(
        &mut self,
        subject: &Expression,
        arms: &[Arm],
        span: Span,
    ) -> Outcome<(InstructionKind, bool)> {
        let (node, subject_type) = self.check_expression(subject, None)?;
        let Type::Union(union_name) = &subject_type else {
            return Err(Report::not_designed(
                subject.span,
                format!("`match` on `{subject_type}`, which isn't a union"),
                "",
            ));
        };
        let slot = match &subject.kind {
            ExpressionKind::Variable(name) => self.find_variable(name),
            _ => None,
        };
        let cases: Vec<String> = self
            .unions
            .get(union_name)
            .map(|info| info.cases.iter().map(|(case, _)| case.clone()).collect())
            .unwrap_or_default();
        let mut covered: Vec<usize> = Vec::new();
        let mut code = Vec::new();
        let mut branches = Vec::new();
        let mut all_end = !arms.is_empty();
        for arm in arms {
            let text = self.through_aliases(&self.canonical(&arm.pattern.text));
            let text = text.strip_prefix("my::").unwrap_or(&text).to_string();
            let Some((pattern_union, case_name)) = text.split_once('.') else {
                if cases.contains(&text) {
                    return Err(Report::refused(
                        arm.pattern.span,
                        "a case says its union",
                        "a union's cases are narrowings, such as `status.failed` (D9, D33)",
                    )
                    .fix(format!("{union_name}.{text}")));
                }
                return Err(Report::not_designed(
                    arm.pattern.span,
                    format!("the pattern `{text}`"),
                    "",
                ));
            };
            if pattern_union != union_name {
                return Err(Report::refused(
                    arm.pattern.span,
                    format!("this `match` is on `{union_name}`, not `{pattern_union}`"),
                    "`match` handles a union's cases (D9)",
                ));
            }
            let case = self.find_case(union_name, case_name).ok_or_else(|| {
                Report::refused(
                    arm.pattern.span,
                    format!("`{union_name}` has no case `{case_name}`"),
                    "a union's cases are narrowings (D33)",
                )
            })?;
            if covered.contains(&case) {
                return Err(Report::not_designed(
                    arm.pattern.span,
                    format!("a `match` with `{text}` twice"),
                    "",
                ));
            }
            covered.push(case);
            let root = root_variable(subject);
            let ((binding_slot, body), ends, branch) = self.on_a_branch(|this| {
                if let Some(state) = slot.and_then(|slot| this.context.state.get_mut(slot)) {
                    state.matched = true;
                }
                this.context.scopes.push(HashMap::new());
                let binding_slot = match &arm.binding {
                    Some(name) => {
                        Some(this.declare_case_name(name, &subject_type, case, root.as_deref())?)
                    }
                    None => None,
                };
                let (body, ends) = this.check_block(&arm.body)?;
                this.end_held_borrows();
                this.context.scopes.pop();
                Ok(((binding_slot, body), ends))
            })?;
            code.push(ir::MatchArm {
                case,
                slot: binding_slot,
                body,
            });
            if !ends {
                branches.push(branch);
                all_end = false;
            }
        }
        let missing: Vec<String> = cases
            .iter()
            .enumerate()
            .filter(|(index, _)| !covered.contains(index))
            .map(|(_, case)| format!("`{union_name}.{case}`"))
            .collect();
        if !missing.is_empty() {
            return Err(Report::refused(
                span,
                format!("this `match` misses {}", missing.join(" and ")),
                "`match` must cover every case (D9)",
            ));
        }
        if !branches.is_empty() {
            self.join_states(&branches);
        }
        if let Some(state) = slot.and_then(|slot| self.context.state.get_mut(slot)) {
            state.matched = false;
        }
        Ok((
            InstructionKind::Match {
                subject: node,
                arms: code,
            },
            all_end,
        ))
    }

    /// Opens a loop for `break` to find. A name is taken while a loop around it has it, which
    /// has no stated answer, like declaring a variable again.
    fn enter_loop(&mut self, name: Option<&crate::ast::Name>) -> Outcome<usize> {
        if let Some(name) = name {
            if self
                .context
                .loops
                .iter()
                .any(|outer| outer.name.as_deref() == Some(name.text.as_str()))
            {
                return Err(Report::not_designed(
                    name.span,
                    format!(
                        "naming a loop `{}` inside another with that name",
                        name.text
                    ),
                    "",
                ));
            }
        }
        let id = self.context.next_loop_id;
        self.context.next_loop_id += 1;
        self.context.loops.push(LoopInfo {
            id,
            name: name.map(|name| name.text.clone()),
            scope_depth: self.context.scopes.len(),
            break_states: Vec::new(),
        });
        Ok(id)
    }

    /// Closes a loop, and says what its `break`s left.
    fn leave_loop(&mut self) -> Vec<Vec<VariableState>> {
        self.context
            .loops
            .pop()
            .map(|finished| finished.break_states)
            .unwrap_or_default()
    }

    /// A test's trust ends with its block, even when a `break` is how the block was left. (D9)
    fn restore_trust(&mut self, before: &[VariableState]) {
        for (state, old) in self.context.state.iter_mut().zip(before) {
            if old.trust.is_none() {
                state.trust = None;
                state.trust_ended = old.trust_ended;
            }
        }
    }

    /// trace: D128
    #[allow(clippy::too_many_lines)] // A range and the three ways through a collection.
    fn check_for(
        &mut self,
        loop_name: Option<&crate::ast::Name>,
        name: &crate::ast::Name,
        source: &ForSource,
        body: &[Statement],
    ) -> Outcome<InstructionKind> {
        self.refuse_second_declaration(name)?;
        let before = self.context.state.clone();
        let (range, collection, item_type, read) = match source {
            ForSource::Collection(collection) => {
                let (node, item_type, read) = self.check_for_collection(collection)?;
                (None, Some(node), item_type, read)
            }
            ForSource::Range(range) => {
                let (nodes, item_type) = self.check_range(range)?;
                (
                    Some(nodes),
                    None,
                    item_type,
                    LoopSource::Read(String::new(), Vec::new()),
                )
            }
        };
        let loop_id = self.enter_loop(loop_name)?;
        self.context.scopes.push(HashMap::new());
        // trace: D180, D182
        let (mutable, origin, must_give) = match &read {
            LoopSource::Read(..) => (false, Origin::ForItem, false),
            LoopSource::Change(_) => (true, Origin::LentMutParameter, false),
            LoopSource::Take { .. } => (false, Origin::Owned, self.is_must_give(&item_type)),
        };
        // While a `for` hands out `must_Give` items, the ones it hasn't reached are still a duty,
        // so leaving early is caught the way a forgotten one is.
        if let LoopSource::Take { name: list, given } = &read {
            if must_give {
                self.declare_hidden(Variable {
                    name: list.clone(),
                    held_type: item_type.clone(),
                    mutable: false,
                    origin: Origin::LoopRest,
                    declared: *given,
                    must_give: true,
                    constant: None,
                    borrows: Vec::new(),
                    borrowed_path: Vec::new(),
                });
            }
        }
        let slot = self.declare_variable(Variable {
            name: name.text.clone(),
            held_type: item_type,
            mutable,
            origin,
            declared: name.span,
            must_give,
            constant: None,
            borrows: Vec::new(),
            borrowed_path: Vec::new(),
        });
        // Nothing may lend `$xs` out mutably while `for` is reading it. (D2)
        let reading = match &read {
            LoopSource::Read(root, path) => (root.clone(), path.clone()),
            _ => (String::new(), Vec::new()),
        };
        self.context.being_read.push(reading);
        if let LoopSource::Change(root) = &read {
            self.context.being_changed.push(root.clone());
        }
        let result = self.check_loop_body(&mut |this| {
            // Each round has its own item, so what one round did to it doesn't carry over.
            if let Some(state) = this.context.state.get_mut(slot) {
                *state = super::VariableState::default();
            }
            let (code, terminated) = this.check_block(body)?;
            // trace: D180
            if must_give && !terminated {
                this.check_duty_ended(slot, None)?;
            }
            Ok((code, terminated))
        });
        if matches!(read, LoopSource::Change(_)) {
            self.context.being_changed.pop();
        }
        self.context.being_read.pop();
        self.context.scopes.pop();
        let code = result?;
        let mut exits = self.leave_loop();
        if !exits.is_empty() {
            exits.push(self.context.state.clone());
            self.join_states(&exits);
        }
        self.restore_trust(&before);
        Ok(match (range, collection) {
            (Some((from, end, included, step)), _) => InstructionKind::ForRange {
                loop_id,
                slot,
                range: Box::new(ir::RangeCode {
                    from,
                    end,
                    included,
                    step,
                }),
                body: code,
            },
            (None, Some(LoopCollection::Value(collection))) => InstructionKind::For {
                loop_id,
                slot,
                collection,
                body: code,
            },
            (None, Some(LoopCollection::Place(place))) => InstructionKind::ForInPlace {
                loop_id,
                slot,
                place,
                body: code,
            },
            (None, None) => InstructionKind::Block(code),
        })
    }

    /// A variable no program can name, such as what a loop still owes. (D180)
    fn declare_hidden(&mut self, variable: Variable) -> usize {
        let slot = self.context.variables.len();
        let key = format!(" hidden {slot}");
        self.context.variables.push(variable);
        self.context
            .state
            .resize(slot + 1, VariableState::default());
        if let Some(scope) = self.context.scopes.last_mut() {
            scope.insert(key, slot);
        }
        slot
    }

    /// `for $r in $xs`, `Lend.mut $xs` or `Give $xs`, and what each does to `$xs`.
    /// trace: D2, D180, D182
    fn check_for_collection(
        &mut self,
        collection: &Expression,
    ) -> Outcome<(LoopCollection, Type, LoopSource)> {
        let ExpressionKind::Hand { word, value } = &collection.kind else {
            let (node, item_type, (root, path)) = self.check_for_read(collection)?;
            return Ok((
                LoopCollection::Value(node),
                item_type,
                LoopSource::Read(root, path),
            ));
        };
        let shown = self.text_of(value.span).to_string();
        match word {
            HandWord::Lend => Err(Report::refused(
                collection.span,
                "a bare `for` already reads in place",
                "`for $x in $xs:` reads each item, so `Lend` adds nothing (D182)",
            )
            .fix(shown)),
            HandWord::Copy => Err(Report::not_designed(
                collection.span,
                "`Copy` on a loop's collection",
                "",
            )),
            HandWord::LendMut | HandWord::Give => {
                if !matches!(value.kind, ExpressionKind::Variable(_))
                    && root_variable(value).is_some()
                {
                    return Err(Report::not_designed(
                        value.span,
                        format!("whether `for` over `{shown}` uses all of what holds it"),
                        "12.1",
                    ));
                }
                let held = if *word == HandWord::Give {
                    self.check_give(value, collection.span)?
                } else {
                    self.check_lent_place(value, HandWord::LendMut)?
                };
                let item_type = Self::loop_item_type(&held.value_type, value.span)?;
                if *word == HandWord::Give {
                    return Ok((
                        LoopCollection::Value(held.node),
                        item_type,
                        LoopSource::Take {
                            name: held.root_name,
                            given: collection.span,
                        },
                    ));
                }
                let Some(place) = held.place else {
                    return Err(Report::refused(
                        value.span,
                        "a constant can't be lent with `Lend.mut`",
                        "a constant's value is fixed at compile time (D83)",
                    ));
                };
                Ok((
                    LoopCollection::Place(place),
                    item_type,
                    LoopSource::Change(held.root_name),
                ))
            }
        }
    }

    /// What a collection's items are, for `for`.
    fn loop_item_type(collection_type: &Type, span: Span) -> Outcome<Type> {
        match collection_type {
            Type::List { item, .. } | Type::Array { item, .. } => Ok((**item).clone()),
            Type::Map { .. } => Err(Report::not_designed(span, "going through a map", "")),
            Type::New { .. } => Err(Report::not_designed(
                span,
                "going through a `type.new`, rather than crossing with `as` first",
                "12.1",
            )),
            other => Err(Report::refused(
                span,
                format!("`for` goes through a collection or range, and this is `{other}`"),
                "`for` goes through a collection or range, reading it in place (D2, D128)",
            )),
        }
    }

    fn check_for_read(
        &mut self,
        collection: &Expression,
    ) -> Outcome<(Node, Type, (String, Vec<usize>))> {
        let (node, collection_type) = self.check_expression(collection, None)?;
        // Items of any type are read in place. (D180)
        let item_type = Self::loop_item_type(&collection_type, collection.span)?;
        let root = root_variable(collection).unwrap_or_default();
        let read_path = if root.is_empty() {
            Vec::new()
        } else {
            self.check_held(collection)?
                .place
                .map(|place| place.keys())
                .unwrap_or_default()
        };
        Ok((node, item_type, (root, read_path)))
    }

    /// A range's numbers all have one type, and one that can never reach its end is refused.
    /// trace: D128
    fn check_range(&mut self, range: &RangeExpression) -> Outcome<(RangeNodes, Type)> {
        let (from, end) = (&range.from, &range.end.value);
        let ((from_node, from_type), (end_node, end_type)) = self.check_two_numbers(from, end)?;
        let kind = Self::range_number(&from_type, from.span)?;
        let end_kind = Self::range_number(&end_type, end.span)?;
        if kind != end_kind {
            return Err(Report::refused(
                range.span,
                format!("this range mixes `{from_type}` and `{end_type}`"),
                super::WHY_TYPES,
            ));
        }
        // trace: D142
        let (from_items, end_items) = (is_in_items(&from_type), is_in_items(&end_type));
        let plain_variable = if from_items && !end_items && !is_number_literal(end) {
            Some(end)
        } else if end_items && !from_items && !is_number_literal(from) {
            Some(from)
        } else {
            None
        };
        if let Some(plain) = plain_variable {
            return Err(Report::not_designed(
                plain.span,
                "a range mixing a plain variable with a number in `items`",
                "",
            ));
        }
        let step_node = match &range.step {
            Some(step) => Some(self.check_range_step(step)?),
            None => None,
        };
        let step_value = match &range.step {
            Some(step) => literal_number(step),
            None => Some(1),
        };
        // A written step of 0 can't reach any end, whatever the numbers turn out to be.
        // trace: D128, D145
        let never_ends = step_value == Some(0);
        let wrong_way = matches!(
            (literal_number(from), literal_number(end), step_value),
            (Some(start), Some(stop), Some(by))
                if (by > 0 && start > stop) || (by < 0 && start < stop)
        );
        {
            if never_ends || wrong_way {
                return Err(Report::refused(
                    range.span,
                    "this range can never reach its end",
                    "a range that can never reach its end is refused, and a negative `step_by:` counts down (D128)",
                ));
            }
        }
        // A literal takes its type from where it lands, so one in `items` makes them all so.
        let unit = (from_items || end_items).then(|| "items".to_string());
        Ok((
            (from_node, end_node, range.end.included, step_node),
            Type::Integer {
                kind,
                bound: None,
                unit,
            },
        ))
    }

    fn range_number(found: &Type, span: Span) -> Outcome<crate::types::IntegerKind> {
        match found {
            Type::Integer {
                kind, unit: None, ..
            } => Ok(*kind),
            // trace: D142
            Type::Integer {
                kind,
                unit: Some(unit),
                ..
            } if unit == "items" => Ok(*kind),
            Type::Integer { .. } => Err(Report::not_designed(
                span,
                "a range of numbers with a unit other than `items`",
                "",
            )),
            Type::Float { .. } => Err(Report::not_designed(span, "a range of floats", "")),
            other => Err(Report::refused(
                span,
                format!("a range is a sequence of numbers, and this is `{other}`"),
                "a range is a sequence of numbers (D128)",
            )),
        }
    }

    /// `step_by:` takes any integer, since a step isn't stored, and `$i` keeps the range's type.
    /// trace: D146
    fn check_range_step(&mut self, step: &Expression) -> Outcome<Node> {
        if let Some(value) = literal_number(step) {
            if let ExpressionKind::WithUnit { unit, .. } = &step.kind {
                if unit.text != "items" {
                    return Err(Report::not_designed(
                        unit.span,
                        "a range of numbers with a unit other than `items`",
                        "",
                    ));
                }
            }
            return Ok(Node {
                span: step.span,
                kind: NodeKind::Constant(Value::Integer(value)),
            });
        }
        let (node, found) = self.check_expression(step, None)?;
        Self::range_number(&found, step.span)?;
        Ok(node)
    }

    /// trace: D128
    fn check_while(
        &mut self,
        condition: &Expression,
        body: &[Statement],
    ) -> Outcome<InstructionKind> {
        let before = self.context.state.clone();
        let loop_id = self.enter_loop(None)?;
        let condition_node = self.check_condition(condition)?;
        let mut exits = vec![self.context.state.clone()];
        let (code, ends, after_once) = self.on_a_branch(|this| this.check_block(body))?;
        if !ends {
            // The second time round, the condition and the body see what the first time did.
            let current = self.context.state.clone();
            self.join_states(&[current, after_once]);
            self.check_condition(condition)?;
            exits.push(self.context.state.clone());
            self.on_a_branch(|this| this.check_block(body))?;
        }
        exits.extend(self.leave_loop());
        self.join_states(&exits);
        self.restore_trust(&before);
        Ok(InstructionKind::While {
            loop_id,
            condition: condition_node,
            body: code,
        })
    }

    /// A `forever` only finishes through a `break`, so with none, nothing runs past it.
    /// trace: D128
    fn check_forever(&mut self, body: &[Statement]) -> Outcome<(InstructionKind, bool)> {
        let before = self.context.state.clone();
        let loop_id = self.enter_loop(None)?;
        let code = self.check_loop_body(&mut |this| this.check_block(body))?;
        let breaks = self.leave_loop();
        let ends = breaks.is_empty();
        if !ends {
            self.join_states(&breaks);
        }
        self.restore_trust(&before);
        Ok((
            InstructionKind::Forever {
                loop_id,
                body: code,
            },
            ends,
        ))
    }

    /// trace: D128
    fn check_break(
        &mut self,
        name: Option<&crate::ast::Path>,
        span: Span,
    ) -> Outcome<InstructionKind> {
        let wanted = match name {
            Some(path) => Some(loop_name_as_used(path)?),
            None => None,
        };
        let target = match &wanted {
            Some(wanted) => self
                .context
                .loops
                .iter()
                .rposition(|outer| outer.name.as_deref() == Some(wanted.as_str())),
            None => self.context.loops.len().checked_sub(1),
        };
        let Some(target) = target else {
            return Err(match &wanted {
                Some(wanted) => Report::refused(
                    span,
                    format!("there's no loop called `{wanted}` around this"),
                    "`break` with a loop's name stops it and every loop inside it (D128)",
                ),
                None => Report::refused(
                    span,
                    "there's no loop around this for `break` to stop",
                    "`break;` stops the innermost loop (D128)",
                ),
            });
        };
        let depth = self
            .context
            .loops
            .get(target)
            .map_or(0, |found| found.scope_depth);
        // A `must_Give` declared inside what's being left goes away here.
        for scope in self.context.scopes.iter().skip(depth) {
            let mut slots: Vec<usize> = scope.values().copied().collect();
            slots.sort_unstable();
            for slot in slots {
                self.check_duty_ended(slot, Some(span))?;
            }
        }
        let state = self.context.state.clone();
        let Some(found) = self.context.loops.get_mut(target) else {
            return Err(Report::refused(span, "this loop isn't open", "(D128)"));
        };
        found.break_states.push(state);
        Ok(InstructionKind::Break(found.id))
    }

    fn check_return(&mut self, value: &Expression, span: Span) -> Outcome<InstructionKind> {
        let return_type = self.context.return_type.clone();
        let node = if matches!(value.kind, ExpressionKind::Void) {
            if return_type != Type::Void {
                return Err(Report::refused(
                    value.span,
                    format!("this function returns `{return_type}`, so it can't `return void`"),
                    "`return` hands back a value (D11)",
                ));
            }
            Node {
                span: value.span,
                kind: NodeKind::Constant(Value::Void),
            }
        } else {
            if return_type == Type::Void {
                return Err(Report::refused(
                    value.span,
                    "this function returns `void`",
                    "a function that returns nothing returns `void` (D11)",
                )
                .fix("void"));
            }
            self.check_into(value, &return_type)?
        };
        for (name, name_span) in self.context.ensures_variables.clone() {
            if let Some(slot) = self.find_variable(&name) {
                if let Some(given) = self.context.state.get(slot).and_then(|state| state.given) {
                    return Err(Report::refused(
                        name_span,
                        format!("this `ensures` reads `${name}`, which was already given away"),
                        "a given value is dead afterwards (D2)",
                    )
                    .because(given, format!("`${name}` was given away here")));
                }
            }
        }
        self.check_duties_at_exit(span)?;
        Ok(InstructionKind::Return(node))
    }

    /// trace: D52
    fn check_fail(
        &mut self,
        path: &crate::ast::Path,
        data: &[crate::ast::FieldValue],
        span: Span,
    ) -> Outcome<InstructionKind> {
        if matches!(
            self.context.kind,
            FunctionKind::Pure | FunctionKind::PureBaked
        ) {
            return Err(Report::not_designed(
                span,
                "whether a `fn.pure` may fail",
                "12.5",
            ));
        }
        if self.context.kind == FunctionKind::Main {
            return Err(Report::not_designed(span, "`fail` in `fn.main`", "12.6"));
        }
        let Some(listed) = self.context.failures.clone() else {
            return Err(Report::refused(
                path.span,
                format!(
                    "this function lists no failures, so it can't `fail {}`",
                    path.text
                ),
                "a function that can fail lists its failures in its body (D52)",
            ));
        };
        let used = failure_name_as_used(path)?;
        refuse_ambiguous_failure(&listed, path)?;
        let failure = find_failure(&listed, &used).ok_or_else(|| {
            Report::refused(
                path.span,
                format!("`{}` isn't in this function's `fails`", path.text),
                "a function that can fail lists its failures in its body (D52)",
            )
        })?;
        let mut nodes = Vec::new();
        for (index, (field_name, field_type)) in failure.data.iter().enumerate() {
            let Some(field) = data.iter().find(|given| given.name.text == *field_name) else {
                return Err(Report::refused(
                    span,
                    format!("`{}` needs `{field_name}`", failure.short),
                    "a failure can carry data in `[ ]` (D47)",
                ));
            };
            if data.get(index).map(|given| &given.name.text) != Some(field_name) {
                return Err(Report::not_designed(
                    field.name.span,
                    "a failure's data written out of order",
                    "",
                ));
            }
            nodes.push(self.check_into(&field.value, field_type)?);
        }
        if let Some(extra) = data.iter().find(|given| {
            !failure
                .data
                .iter()
                .any(|(name, _)| *name == given.name.text)
        }) {
            return Err(Report::refused(
                extra.name.span,
                format!("`{}` has no `{}`", failure.short, extra.name.text),
                "a failure can carry data in `[ ]` (D47)",
            ));
        }
        self.check_duties_at_exit(span)?;
        Ok(InstructionKind::Fail {
            name: failure.full.clone(),
            shown: failure.short.clone(),
            data: nodes,
        })
    }

    fn check_expression_statement(&mut self, value: &Expression) -> Outcome<InstructionKind> {
        let unwrapped = match &value.kind {
            ExpressionKind::Try(inner)
            | ExpressionKind::Fallback { value: inner, .. }
            | ExpressionKind::Otherwise { value: inner, .. } => inner.as_ref(),
            _ => value,
        };
        if !matches!(
            unwrapped.kind,
            ExpressionKind::Call { .. }
                | ExpressionKind::Method { .. }
                | ExpressionKind::Builtin(_)
        ) {
            if matches!(
                value.kind,
                ExpressionKind::Text(_)
                    | ExpressionKind::Integer { .. }
                    | ExpressionKind::Float { .. }
                    | ExpressionKind::Empty
                    | ExpressionKind::Variable(_)
            ) {
                return Err(Report::not_designed(
                    value.span,
                    "what an arm's last line means, such as a value on its own",
                    "12.6",
                ));
            }
            return Err(Report::not_designed(
                value.span,
                "a value on its own as a statement",
                "",
            ));
        }
        let (node, value_type) = self.check_statement_call(value)?;
        if value_type != Type::Void {
            return Err(Report::not_designed(
                value.span,
                format!("dropping a value of `{value_type}` by not using it"),
                "",
            ));
        }
        Ok(InstructionKind::Evaluate(node))
    }

    /// Reading `$n` only for its truth, as `if`, `requires` and `ensures` do.
    pub(super) fn check_condition(&mut self, condition: &Expression) -> Outcome<Node> {
        let (node, condition_type) = self.check_expression(condition, Some(&Type::Bool))?;
        if condition_type != Type::Bool {
            return Err(Report::refused(
                condition.span,
                format!("this is `{condition_type}`, where a `bool` is needed"),
                super::WHY_TYPES,
            ));
        }
        Ok(node)
    }

    /// `$f` in `case status.failed $f:`: the case's data, which it trusts all through its arm.
    /// It's a read-only view of what's matched, so changing that ends it, like a stored borrow.
    /// trace: D139
    fn declare_case_name(
        &mut self,
        name: &crate::ast::Name,
        subject_type: &Type,
        case: usize,
        root: Option<&str>,
    ) -> Outcome<usize> {
        self.refuse_second_declaration(name)?;
        let slot = self.declare_variable(Variable {
            name: name.text.clone(),
            held_type: subject_type.clone(),
            mutable: false,
            origin: Origin::StoredBorrow,
            declared: name.span,
            must_give: false,
            constant: None,
            borrows: root.map(str::to_string).into_iter().collect(),
            borrowed_path: Vec::new(),
        });
        if let Some(state) = self.context.state.get_mut(slot) {
            state.trust = Some(case);
        }
        Ok(slot)
    }

    /// `instead 0;`: the value used in place of the failed call. (D139)
    /// trace: D139
    fn check_instead(&mut self, value: &Expression, span: Span) -> Outcome<InstructionKind> {
        const WHY: &str = "`instead` gives the value used in place of a failed call (D139)";
        let Some(arm) = self.context.arms.last().cloned() else {
            return Err(Report::refused(
                span,
                "`instead` goes in a failure's `case`, and there's none around this",
                WHY,
            ));
        };
        if self.context.loops.len() != arm.loop_depth {
            return Err(Report::not_designed(
                span,
                "`instead` inside a loop inside its arm",
                "",
            ));
        }
        if arm.value_type == Type::Void {
            return Err(Report::refused(
                span,
                "this call gives back nothing, so there's nothing to use instead",
                WHY,
            ));
        }
        let node = self.check_into(value, &arm.value_type)?;
        for scope in self.context.scopes.iter().skip(arm.scope_depth) {
            let mut slots: Vec<usize> = scope.values().copied().collect();
            slots.sort_unstable();
            for slot in slots {
                self.check_duty_ended(slot, Some(span))?;
            }
        }
        let state = self.context.state.clone();
        if let Some(found) = self.context.arms.last_mut() {
            found.instead_states.push(state);
        }
        Ok(InstructionKind::Instead(node))
    }

    /// trace: D46, D139
    #[allow(clippy::too_many_lines)] // Every rule an arm has, top to bottom.
    pub(super) fn check_arms(
        &mut self,
        arms: &[Arm],
        failures: &[Failure],
        value_type: &Type,
        span: Span,
    ) -> Outcome<Vec<ir::FailureArm>> {
        let mut code = Vec::new();
        let mut covered: Vec<String> = Vec::new();
        let before = self.context.state.clone();
        let mut branches = vec![before];
        for arm in arms {
            let used = failure_name_as_used(&arm.pattern)?;
            refuse_ambiguous_failure(failures, &arm.pattern)?;
            let Some(failure) = find_failure(failures, &used) else {
                if self.failure_sets.contains_key(&arm.pattern.text) {
                    return Err(Report::not_designed(
                        arm.pattern.span,
                        "one arm covering a whole named set",
                        "",
                    ));
                }
                return Err(Report::refused(
                    arm.pattern.span,
                    format!("this call can't fail with `{}`", arm.pattern.text),
                    "an arm handles a failure still possible at that call (D46)",
                ));
            };
            if covered.contains(&failure.full) {
                return Err(Report::not_designed(
                    arm.pattern.span,
                    format!("two arms for `{}`", failure.short),
                    "",
                ));
            }
            covered.push(failure.full.clone());
            // trace: D139
            if *value_type != Type::Void {
                if let Some(StatementKind::Expression(last)) =
                    arm.body.last().map(|statement| &statement.kind)
                {
                    if !matches!(
                        last.kind,
                        ExpressionKind::Call { .. }
                            | ExpressionKind::Method { .. }
                            | ExpressionKind::Try(_)
                            | ExpressionKind::Fallback { .. }
                            | ExpressionKind::Otherwise { .. }
                    ) {
                        let shown = self.text_of(last.span);
                        return Err(Report::refused(
                            last.span,
                            "a value on its own reads like a leftover",
                            "`instead` gives the value used in place of a failed call (D139)",
                        )
                        .fix(format!("instead {shown}")));
                    }
                }
            }
            let failure = failure.clone();
            self.context.arms.push(ArmInfo {
                value_type: value_type.clone(),
                scope_depth: self.context.scopes.len(),
                loop_depth: self.context.loops.len(),
                instead_states: Vec::new(),
            });
            let ((binding_slot, body), ends, branch) = self.on_a_branch(|this| {
                this.context.scopes.push(HashMap::new());
                let binding_slot = match &arm.binding {
                    Some(name) => Some(this.declare_failure_name(name, &failure)?),
                    None => None,
                };
                let (body, ends) = this.check_block(&arm.body)?;
                this.context.scopes.pop();
                Ok(((binding_slot, body), ends))
            })?;
            if let Some(finished) = self.context.arms.pop() {
                branches.extend(finished.instead_states);
            }
            if !ends {
                if *value_type != Type::Void {
                    return Err(Report::refused(
                        arm.pattern.span,
                        format!("this arm can run off its end, and `{value_type}` is needed here"),
                        "an arm gives a value with `instead`, or can `return`, `break` or `fail` (D139)",
                    ));
                }
                branches.push(branch);
            }
            code.push(ir::FailureArm {
                failure: failure.full.clone(),
                slot: binding_slot,
                body,
            });
        }
        let missing: Vec<String> = failures
            .iter()
            .filter(|failure| !covered.contains(&failure.full))
            .map(|failure| format!("`{}`", failure.short))
            .collect();
        if !missing.is_empty() {
            return Err(Report::refused(
                span,
                format!("this doesn't handle {}", missing.join(" and ")),
                "the arms must cover every failure still possible at that call (D46)",
            ));
        }
        self.join_states(&branches);
        Ok(code)
    }

    /// `$f` in `case negative $f:`, whose fields are the failure's data. (D139)
    fn declare_failure_name(
        &mut self,
        name: &crate::ast::Name,
        failure: &Failure,
    ) -> Outcome<usize> {
        self.refuse_second_declaration(name)?;
        Ok(self.declare_variable(Variable {
            name: name.text.clone(),
            held_type: Type::Failure {
                name: failure.short.clone(),
                fields: failure.data.clone(),
            },
            mutable: false,
            origin: Origin::ForItem,
            declared: name.span,
            must_give: false,
            constant: None,
            borrows: Vec::new(),
            borrowed_path: Vec::new(),
        }))
    }

    pub(super) fn item_limit_check(count: i128, span: Span) -> Outcome<()> {
        if count > ITEM_LIMIT {
            return Err(Report::not_designed(
                span,
                format!("building {count} items, past the oracle's limit of {ITEM_LIMIT}"),
                "",
            ));
        }
        Ok(())
    }
}

fn is_in_items(found: &Type) -> bool {
    matches!(found, Type::Integer { unit: Some(unit), .. } if unit == "items")
}

/// A loop's name that clashes is written with `my::` wherever it's used. (D75)
fn loop_name_as_used(path: &crate::ast::Path) -> Outcome<String> {
    if let Some(mine) = path.text.strip_prefix("my::") {
        if !is_clashing(mine) {
            return Err(Report::not_designed(
                path.span,
                format!("`my::` on `{mine}`, which doesn't clash"),
                "",
            ));
        }
        return Ok(mine.to_string());
    }
    // trace: D75
    if is_clashing(&path.text) {
        return Err(Report::refused(
            path.span,
            format!("`{}` is built in", path.text),
            "a name of yours that clashes is written with `my::` (D75)",
        )
        .fix(format!("my::{}", path.text)));
    }
    Ok(path.text.clone())
}

/// A failure's name that clashes is written with `my::` wherever it's used. (D75)
fn failure_name_as_used(path: &crate::ast::Path) -> Outcome<String> {
    if let Some(mine) = path.text.strip_prefix("my::") {
        if !is_clashing(mine) {
            return Err(Report::not_designed(
                path.span,
                format!("`my::` on `{mine}`, which doesn't clash"),
                "",
            ));
        }
        return Ok(mine.to_string());
    }
    // trace: D75
    if is_clashing(&path.text) {
        return Err(Report::refused(
            path.span,
            format!("`{}` is built in", path.text),
            "a name of yours that clashes is written with `my::` (D75)",
        )
        .fix(format!("my::{}", path.text)));
    }
    Ok(path.text.clone())
}

/// A short name two failures share says neither of them. (D47)
fn refuse_ambiguous_failure(failures: &[Failure], path: &crate::ast::Path) -> Outcome<()> {
    let matching: Vec<&Failure> = failures
        .iter()
        .filter(|failure| failure.short == path.text || failure.full == path.text)
        .collect();
    if matching.len() > 1 {
        return Err(Report::refused(
            path.span,
            format!("two failures here are called `{}`", path.text),
            "two sets sharing a failure's name need the full name (D47)",
        ));
    }
    Ok(())
}

/// A failure by the name a list or arm uses: its short name, or its full one.
pub(super) fn find_failure<'f>(failures: &'f [Failure], name: &str) -> Option<&'f Failure> {
    failures
        .iter()
        .find(|failure| failure.short == name || failure.full == name)
}

/// The variable a place starts from, if the expression is one.
pub(super) fn root_variable(expression: &Expression) -> Option<String> {
    match &expression.kind {
        ExpressionKind::Variable(name) => Some(name.clone()),
        ExpressionKind::Field { base, .. } | ExpressionKind::Item { base, .. } => {
            root_variable(base)
        }
        ExpressionKind::Group(inner) => root_variable(inner),
        _ => None,
    }
}

/// Every `$name` an expression reads, in order, once each.
pub(super) fn collect_variables(expression: &Expression, out: &mut Vec<(String, Span)>) {
    walk(expression, &mut |part| {
        if let ExpressionKind::Variable(name) = &part.kind {
            if !out.iter().any(|(seen, _)| seen == name) {
                out.push((name.clone(), part.span));
            }
        }
    });
}

fn mentions_result(expression: &Expression) -> bool {
    let mut found = false;
    walk(expression, &mut |part| {
        if let ExpressionKind::Path(path) = &part.kind {
            found = found || path.text == "result";
        }
    });
    found
}

/// Visits an expression and everything inside it.
pub(super) fn walk(expression: &Expression, visit: &mut dyn FnMut(&Expression)) {
    visit(expression);
    match &expression.kind {
        ExpressionKind::All(inner)
        | ExpressionKind::Not(inner)
        | ExpressionKind::Negate(inner)
        | ExpressionKind::BitNot(inner)
        | ExpressionKind::Try(inner)
        | ExpressionKind::Bake(inner)
        | ExpressionKind::Group(inner) => walk(inner, visit),
        ExpressionKind::Items(items) => {
            for item in items {
                walk(item, visit);
            }
        }
        ExpressionKind::WithUnit { number, .. } => walk(number, visit),
        ExpressionKind::Field { base, .. } => walk(base, visit),
        ExpressionKind::Item { base, position, .. } => {
            walk(base, visit);
            walk(position, visit);
        }
        ExpressionKind::Build { fields, .. } => {
            for field in fields {
                walk(&field.value, visit);
            }
        }
        ExpressionKind::Call { arguments, .. } => {
            for argument in arguments {
                walk(&argument.value, visit);
            }
        }
        ExpressionKind::Method {
            receiver,
            arguments,
            ..
        } => {
            walk(receiver, visit);
            for argument in arguments {
                walk(&argument.value, visit);
            }
        }
        ExpressionKind::Binary { left, right, .. } => {
            walk(left, visit);
            walk(right, visit);
        }
        ExpressionKind::Compare { first, rest } => {
            walk(first, visit);
            for (_, value) in rest {
                walk(value, visit);
            }
        }
        ExpressionKind::As { value, .. }
        | ExpressionKind::Hand { value, .. }
        | ExpressionKind::Otherwise { value, .. } => walk(value, visit),
        ExpressionKind::Fallback { value, fallback } => {
            walk(value, visit);
            walk(fallback, visit);
        }
        _ => {}
    }
}

/// The bound a test on a plain float checks, when the test's type is that float with a bound.
/// trace: D158, D20
fn float_test_bound(subject: &Type, test: &Type) -> Option<(f64, f64)> {
    match (subject, test) {
        (
            Type::Float {
                kind,
                bound: None,
                unit,
                rules: FloatRules::Plain,
            },
            Type::Float {
                kind: test_kind,
                bound: Some((low, high)),
                unit: test_unit,
                rules: FloatRules::Plain,
            },
        ) if kind == test_kind && unit == test_unit => Some((low.value(), high.value())),
        _ => None,
    }
}
