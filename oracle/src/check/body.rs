//! Statements, and how ownership and trust flow through them.

use std::collections::HashMap;

use super::hand::UseKind;
use super::{Checker, Failure, FunctionContext, Origin, Variable, VariableState, ITEM_LIMIT};
use crate::ast::{
    Arm, Contract, Expression, ExpressionKind, Function, FunctionKind, ParameterWord, Statement,
    StatementKind, TypeExpression, TypeKind,
};
use crate::ir::{self, Instruction, InstructionKind, Node, NodeKind, Place};
use crate::report::{Outcome, Report, ReportExtra, Span};
use crate::syntax::is_clashing;
use crate::types::{Sources, Type, Value};

/// A block's code, and whether it can't run past its end.
type CheckedBlock = Outcome<(Vec<Instruction>, bool)>;

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
                self.check_into(value, &info.parameter_type)?;
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
        let scope_check = self.check_borrows_at_scope_end().and_then(|()| {
            if terminated {
                Ok(())
            } else {
                self.check_innermost_scope_end(end_span)
            }
        });
        self.context.scopes.pop();
        scope_check?;
        Ok((code, terminated))
    }

    fn check_statements(&mut self, statements: &[Statement]) -> Outcome<(Vec<Instruction>, bool)> {
        let mut code = Vec::new();
        let mut terminated = false;
        for statement in statements {
            let (instruction, ends) = self.check_statement(statement)?;
            code.push(instruction);
            terminated = terminated || ends;
        }
        Ok((code, terminated))
    }

    /// A stored borrow whose source got used, and which was never read again, is only wrong if a
    /// borrow lasts to the end of its scope, and that isn't decided. (D2)
    fn check_borrows_at_scope_end(&self) -> Outcome<()> {
        let Some(scope) = self.context.scopes.last() else {
            return Ok(());
        };
        let mut slots: Vec<usize> = scope.values().copied().collect();
        slots.sort_unstable();
        for slot in slots {
            let used_at = self
                .context
                .state
                .get(slot)
                .and_then(|state| state.invalidated);
            if let Some(used_at) = used_at {
                return Err(Report::not_designed(
                    used_at,
                    "whether a stored borrow ends at its last use or at the end of its scope",
                    "",
                ));
            }
        }
        Ok(())
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

    /// A `must_Give` value has to have been given by the time its holder goes away. (D2)
    fn check_duty_ended(&self, slot: usize, exit: Option<Span>) -> Outcome<()> {
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
        if variable.origin == Origin::GivenParameter && given.is_none() {
            return Err(Report::not_designed(
                variable.declared,
                format!(
                    "how the `must_Give` duty of `${}` ends in the function that receives it",
                    variable.name
                ),
                "12.8",
            ));
        }
        let why = "a `must_Give` value is handed off with `Give` exactly once (D2)";
        if let Some(given_at) = given {
            return Err(Report::refused(
                variable.declared,
                format!("`${}` isn't given away on every path", variable.name),
                why,
            )
            .because(
                given_at,
                format!("`${}` was given away here, on only one path", variable.name),
            ));
        }
        let report = match exit {
            Some(exit_span) => Report::refused(
                exit_span,
                format!("this leaves with `${}` never given away", variable.name),
                why,
            )
            .because(
                variable.declared,
                format!("`${}` was made here", variable.name),
            ),
            None => Report::refused(
                variable.declared,
                format!("`${}` is never given away", variable.name),
                why,
            ),
        };
        Err(report)
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
                some_path = some_path || state.given_on_some_path;
            }
            slot_state.given = given;
            slot_state.given_on_some_path =
                some_path || (given_count > 0 && given_count < branch_count);
            slot_state.invalidated = invalidated;
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
            StatementKind::If { condition, body } => {
                let condition = self.check_condition(condition)?;
                let before = self.context.state.clone();
                let (body, ends, branch) = self.on_a_branch(|this| this.check_block(body))?;
                if !ends {
                    self.join_states(&[before, branch]);
                }
                InstructionKind::If { condition, body }
            }
            StatementKind::Test {
                subject,
                written_type,
                body,
            } => self.check_test(subject, written_type, body)?,
            StatementKind::Match { subject, arms } => {
                let (kind, ends) = self.check_match(subject, arms, span)?;
                terminated = ends;
                kind
            }
            StatementKind::For {
                name,
                collection,
                body,
            } => self.check_for(name, collection, body)?,
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
        let held_type = self.resolve_type(written_type)?;
        if let TypeKind::StoredBorrow { .. } = &written_type.kind {
            return self.check_stored_borrow(mutable, name, &held_type, value);
        }
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
        });
        Ok(InstructionKind::Let { slot, value: node })
    }

    /// trace: D2
    fn check_stored_borrow(
        &mut self,
        mutable: bool,
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
        if place.root_name.is_empty() || !names.contains(&place.root_name) {
            return Err(Report::refused(
                value.span,
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
        if place.value_type != **inner {
            return Err(Self::mismatch(value.span, inner, &place.value_type));
        }
        let slot = self.declare_variable(Variable {
            name: name.text.clone(),
            held_type: held_type.clone(),
            mutable: *borrow_mutable,
            origin: Origin::StoredBorrow,
            declared: name.span,
            must_give: false,
            constant: None,
            borrows: names.clone(),
        });
        let Some(source) = place.place else {
            return Err(Report::not_designed(
                value.span,
                "a stored borrow of a constant",
                "12.8",
            ));
        };
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
            if matches!(target.kind, ExpressionKind::Field { .. }) {
                return Err(Report::not_designed(target.span, "`change` on a field", ""));
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
        self.refuse_while_read(name, target.span)?;
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

    /// `if $s: status.failed:` and `if $n: i32[min: 0, max: 100]:`. (D9, D20)
    #[allow(clippy::too_many_lines)] // Unions and integers each have their own rules here.
    fn check_test(
        &mut self,
        subject: &Expression,
        written_type: &TypeExpression,
        body: &[Statement],
    ) -> Outcome<InstructionKind> {
        let ExpressionKind::Variable(name) = &subject.kind else {
            return Err(Report::not_designed(
                subject.span,
                "the test on anything but a variable",
                "12.4",
            ));
        };
        let (node, subject_type) = self.check_expression(subject, None)?;
        let slot = self.find_variable(name).ok_or_else(|| {
            Report::refused(subject.span, format!("`${name}` isn't declared"), "(D2)")
        })?;
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
            if self
                .context
                .state
                .get(slot)
                .is_some_and(|state| state.trust.is_some())
            {
                return Err(Report::not_designed(
                    written_type.span,
                    "the test on a value already known to be that type",
                    "12.2",
                ));
            }
            let before = self.context.state.clone();
            let (code, ends, branch) = self.on_a_branch(|this| {
                if let Some(state) = this.context.state.get_mut(slot) {
                    state.trust = Some(case);
                    state.trust_ended = None;
                }
                this.check_block(body)
            })?;
            if !ends {
                self.join_states(&[before, branch]);
            }
            // The trust ends at `end if`, so a read after it is the plain "outside the block" error.
            if let Some(state) = self.context.state.get_mut(slot) {
                state.trust = None;
                state.trust_ended = None;
            }
            return Ok(InstructionKind::TestCase {
                subject: node,
                case,
                body: code,
            });
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
                let before = self.context.state.clone();
                let (code, ends, branch) = self.on_a_branch(|this| this.check_block(body))?;
                if !ends {
                    self.join_states(&[before, branch]);
                }
                return Ok(InstructionKind::TestRange {
                    subject: node,
                    low: *low,
                    high: *high,
                    body: code,
                });
            }
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
        let text = self.canonical(&path.text);
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
            let text = self.canonical(&arm.pattern.text);
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
            let (body, ends, branch) = self.on_a_branch(|this| {
                if let Some(slot) = slot {
                    if let Some(state) = this.context.state.get_mut(slot) {
                        state.trust = Some(case);
                        state.trust_ended = None;
                    }
                }
                this.check_block(&arm.body)
            })?;
            code.push((case, body));
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
            state.trust = None;
            state.trust_ended = None;
        }
        Ok((
            InstructionKind::Match {
                subject: node,
                arms: code,
            },
            all_end,
        ))
    }

    /// trace: D2
    fn check_for(
        &mut self,
        name: &crate::ast::Name,
        collection: &Expression,
        body: &[Statement],
    ) -> Outcome<InstructionKind> {
        self.refuse_second_declaration(name)?;
        let (node, collection_type) = self.check_expression(collection, None)?;
        let item_type = match &collection_type {
            Type::List { item, .. } | Type::Array { item, .. } => (**item).clone(),
            Type::Map { .. } => {
                return Err(Report::not_designed(
                    collection.span,
                    "going through a map",
                    "",
                ));
            }
            other => {
                return Err(Report::refused(
                    collection.span,
                    format!("`for` goes through a collection, and this is `{other}`"),
                    "`for` goes through a collection, reading it in place (D2)",
                ));
            }
        };
        if matches!(
            item_type,
            Type::Struct(_) | Type::Union(_) | Type::Array { .. }
        ) {
            return Err(Report::not_designed(
                collection.span,
                "going through items that are structs, unions or arrays",
                "12.8",
            ));
        }
        let root = root_variable(collection).unwrap_or_default();
        self.context.scopes.push(HashMap::new());
        let slot = self.declare_variable(Variable {
            name: name.text.clone(),
            held_type: item_type,
            mutable: false,
            origin: Origin::ForItem,
            declared: name.span,
            must_give: false,
            constant: None,
            borrows: Vec::new(),
        });
        // Nothing may lend `$xs` out mutably while `for` is reading it. (D2)
        self.context.being_read.push(root);
        let result = self.check_loop_body(&mut |this| this.check_block(body));
        self.context.being_read.pop();
        self.context.scopes.pop();
        Ok(InstructionKind::For {
            slot,
            collection: node,
            body: result?,
        })
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

    /// trace: D46
    pub(super) fn check_arms(
        &mut self,
        arms: &[Arm],
        failures: &[Failure],
        value_is_void: bool,
        span: Span,
    ) -> Outcome<Vec<(String, Vec<Instruction>)>> {
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
            let (body, ends, branch) = self.on_a_branch(|this| this.check_block(&arm.body))?;
            if !ends {
                if !value_is_void {
                    return Err(Report::not_designed(
                        arm.pattern.span,
                        "what an arm that runs off its end gives back",
                        "12.6",
                    ));
                }
                branches.push(branch);
            }
            code.push((failure.full.clone(), body));
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
        ExpressionKind::Field { base, .. } => root_variable(base),
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
