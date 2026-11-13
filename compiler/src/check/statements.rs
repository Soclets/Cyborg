//! Statements, and how variables' states flow through blocks, branches and loops (D203).

use crate::checked::{self, ExpressionKind as Kind, MachineType};
use crate::diagnostic::{Diagnostic, Kind as DiagnosticKind};
use crate::source::Span;
use crate::syntax::{self, Condition, ExpressionKind, StatementKind};

use super::expressions::{CONVERT_WHY, Typed, When, failure, root_name};
use super::types::Type;
use super::{Body, Check, Checker, Loop, NOT_DECIDED, Origin, Reported, State, Variable};

const RANGE_ORDER_WHY: &str =
    "`from:` is required, `to:` or `before:` says where it ends, and `step_by:` comes after them, in that order (D128)";

/// A checked block, and whether every path through it leaves: by `return`, `break`, or a loop that never ends.
pub struct Checked {
    pub block: checked::Block,
    pub leaves: bool,
    /// Whether it leaves by `instead`, which carries on after the call it handles.
    pub gives: bool,
    /// The variables declared straight in it, which an `on.target` branch passes on (D233).
    pub declared: Vec<Variable>,
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
        body.defers.push(Vec::new());
        let count = body.variables.len();
        let mut block = Vec::new();
        let mut leaves = false;
        let mut gives = false;
        // What the statement around this block did so far stays with it, and goes before it, as does what it needs
        // run first, like a task's join in a `let` whose `otherwise:` arms are this block (D376).
        let outer = std::mem::take(&mut body.events);
        let outer_before = std::mem::take(&mut body.before);
        for statement in statements {
            if matches!(statement.kind, StatementKind::Instead(_)) {
                gives = true;
            }
            // Leaving runs every deferred statement it leaves behind, innermost first (D325).
            let leaving_from = match &statement.kind {
                StatementKind::Return(_) | StatementKind::Fail(_) => Some(0),
                StatementKind::Break(label) => Some(
                    body.loops
                        .iter()
                        .rev()
                        .find(|each| {
                            label.as_ref().is_none_or(|label| each.label.as_deref() == Some(label.text.as_str()))
                        })
                        .map_or(0, |each| each.defers),
                ),
                _ => None,
            };
            if let Some(from) = leaving_from {
                self.run_defers(body, from, &mut block);
            }
            body.statement = None;
            let mark = block.len();
            let result = self.statement(body, statement, &mut block);
            // What the statement needs run first, like a waiting task before its `done` (D376).
            let before = std::mem::take(&mut body.before);
            if result.is_ok() {
                for (offset, each) in before.into_iter().enumerate() {
                    block.insert((mark + offset).min(block.len()), each);
                }
            }
            if result.is_ok() && matches!(statement.kind, StatementKind::Return(_) | StatementKind::Fail(_)) {
                body.events.push(crate::flow::Event::Leave { span: statement.span, locals: None });
            }
            let events = std::mem::take(&mut body.events);
            // A refused statement didn't happen, as far as ownership goes.
            if result.is_ok() && !events.is_empty() {
                block.insert(mark.min(block.len()), checked::Statement::Flow(events));
            }
            // A `return` or `break` leaves even when what it hands back is refused.
            if matches!(result, Ok(true))
                || matches!(
                    statement.kind,
                    StatementKind::Return(_)
                        | StatementKind::Break(_)
                        | StatementKind::Fail(_)
                        | StatementKind::Instead(_)
                )
            {
                leaves = true;
            }
        }
        if !leaves {
            let depth = body.defers.len().saturating_sub(1);
            self.run_defers(body, depth, &mut block);
        }
        body.defers.pop();
        let locals: Vec<checked::LocalId> =
            body.variables[count.min(body.variables.len())..].iter().map(|variable| variable.local).collect();
        if !leaves && !locals.is_empty() {
            block.push(checked::Statement::Flow(vec![crate::flow::Event::ScopeEnd { locals }]));
        }
        body.events = outer;
        body.before = outer_before;
        let declared = body
            .scopes
            .pop()
            .unwrap_or_default()
            .iter()
            .filter_map(|(_, index)| body.variables.get(*index).cloned())
            .collect();
        body.forget_after(count);
        Checked { block, leaves, gives, declared }
    }

    /// `defer …;`: the statement is kept for where its block ends, and whatever it gives away is promised to it, so
    /// nothing else gives it away first (D325).
    fn defer_statement(&mut self, body: &mut Body, span: Span, deferred: &syntax::Statement) {
        if let StatementKind::Expression(expression) = &deferred.kind
            && let ExpressionKind::Call { arguments, .. } = &expression.kind
        {
            for argument in arguments {
                if argument.ownership.map(|(word, _)| word) == Some(syntax::Ownership::Give)
                    && let ExpressionKind::Variable(name) = &argument.value.kind
                    && let Some(variable) = body.lookup(&name.text)
                {
                    body.promised.push((variable.local, span));
                }
            }
        }
        if let Some(frame) = body.defers.last_mut() {
            frame.push(super::Deferred::Written(deferred.clone()));
        }
    }

    /// Runs the deferred statements of every open block from `from` in, innermost first and each block's in reverse
    /// (D325). A statement deferred once and run at several exits says each error once.
    pub fn run_defers(&mut self, body: &mut Body, from: usize, block: &mut checked::Block) {
        let frames: Vec<Vec<super::Deferred>> = body.defers.get(from..).map(<[_]>::to_vec).unwrap_or_default();
        if frames.iter().all(Vec::is_empty) {
            return;
        }
        let running = std::mem::replace(&mut body.running_defers, true);
        for frame in frames.iter().rev() {
            for deferred in frame.iter().rev() {
                let deferred = match deferred {
                    super::Deferred::Written(deferred) => deferred,
                    super::Deferred::Unlock(unlock) => {
                        block.push(unlock.clone());
                        continue;
                    }
                };
                let before = self.errors.len();
                let mark = block.len();
                let outer = std::mem::take(&mut body.events);
                let result = self.statement(body, deferred, block);
                let mut events = std::mem::replace(&mut body.events, outer);
                // Its reads and borrows were the code's as written, where `defer` is, and only giving away happens
                // here, where it runs (D325).
                events.retain(|event| matches!(event, crate::flow::Event::Give { .. }));
                if result.is_ok() && !events.is_empty() {
                    block.insert(mark.min(block.len()), checked::Statement::Flow(events));
                }
                let added = self.errors.split_off(before);
                for error in added {
                    if !self.errors.contains(&error) {
                        self.errors.push(error);
                    }
                }
            }
        }
        body.running_defers = running;
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
                        known_case: None,
                    },
                    State::Unassigned,
                );
                // A `decl` of something with a duty isn't decided, though its variable stays, with its type (D203).
                if let Some(variable) = body.lookup(&name.text).cloned()
                    && self.must_give(&variable.value_type)
                {
                    return Err(self.error(Diagnostic::not_designed(
                        written_type.span,
                        format!("`decl` of `{}`, which has a duty or a borrow", variable.value_type),
                        NOT_DECIDED,
                    )));
                }
                Ok(false)
            }
            StatementKind::Assign { target, value } => {
                self.assign(body, statement.span, target, value, block).map(|()| false)
            }
            StatementKind::Change { target, value } => {
                self.change(body, statement.span, target, value, block).map(|()| false)
            }
            StatementKind::Return(value) => {
                if let Some((_, list, given)) = body.give_loops.last().cloned() {
                    return Err(self.leaves_items(statement.span, &list, given));
                }
                self.return_statement(body, statement, value.as_ref(), block).map(|()| true)
            }
            StatementKind::Expression(expression) => {
                body.statement = Some(expression.span);
                self.expression_statement(body, expression, block).map(|()| false)
            }
            StatementKind::If(chain) => self.if_statement(body, chain, block),
            StatementKind::While { label, condition, body: inner, end } => {
                body.loop_opener = opener(self.text, statement.span);
                if label.is_none() {
                    self.check_closer(end, "end while", super::CLOSER_WHY);
                }
                self.while_statement(body, label.as_ref(), condition, inner, block).map(|()| false)
            }
            StatementKind::Forever { label, body: inner, end } => {
                body.loop_opener = opener(self.text, statement.span);
                if label.is_none() {
                    self.check_closer(end, "end forever", super::CLOSER_WHY);
                }
                self.forever(body, label.as_ref(), inner, block)
            }
            StatementKind::Match(choice) => self.match_statement(body, statement.span, choice, block),
            StatementKind::Fail(value) => self.fail_statement(body, statement.span, value, block).map(|()| true),
            StatementKind::Instead(value) => self.instead_statement(body, statement.span, value, block).map(|()| true),
            StatementKind::For(each) => {
                body.loop_opener = opener(self.text, statement.span);
                match &each.label {
                    Some(label) => self.check_closer(&each.end, &format!("end for {}", label.text), super::CLOSER_WHY),
                    None => self.check_closer(
                        &each.end,
                        "end for",
                        "the closer repeats what it closes, including the name (D33, D128)",
                    ),
                }
                if let Some(done) = self.chunk_loop(body, statement.span, each, block) {
                    return done.map(|()| false);
                }
                if let Some((name, _)) = &each.leftovers {
                    // trace: D300
                    return Err(self.error(Diagnostic::refused(
                        name.span,
                        "a `leftovers` section belongs to a `for` over `simd_chunks.leftovers`",
                        "`.leftovers` hands the leftover items to a `leftovers` section, and no other loop has one (D300)",
                    )));
                }
                self.for_statement(body, each, block).map(|()| false)
            }
            StatementKind::Break(label) => {
                self.break_statement(body, statement.span, label.as_ref(), block).map(|()| true)
            }
            // The top of a body is read with the function's header (D114).
            // A default is checked against its parameter's type here, where it's written, whether or not a call uses
            // it (D114).
            StatementKind::Defaults(defaults) => {
                let events = std::mem::take(&mut body.events);
                for default in defaults {
                    let Some(target) = default.name.segments.first() else { continue };
                    let Some(variable) = body.lookup(&target.name.text).cloned() else { continue };
                    // A unit can come from the label, so a default with one is checked where it's used (D144).
                    if matches!(
                        variable.value_type,
                        Type::Integer { unit: Some(_), .. } | Type::Float { unit: Some(_), .. }
                    ) {
                        continue;
                    }
                    let _ = self.value_into(body, &default.value, &variable.value_type);
                }
                body.events = events;
                body.before.clear();
                Ok(false)
            }
            StatementKind::Labels(_) | StatementKind::Requires { .. } | StatementKind::Ensures { .. } => Ok(false),
            StatementKind::Finish(value) => self.finish_statement(body, statement.span, value).map(|()| false),
            StatementKind::Locked { locks, names, body: inner, .. } => {
                self.locked(body, statement.span, (locks, names, inner), block)
            }
            StatementKind::Defer(deferred) => {
                self.defer_statement(body, statement.span, deferred);
                Ok(false)
            }
            StatementKind::Shield(shield) => self.shield_statement(body, shield, block),
            StatementKind::OnTarget(choice) => self.on_target(body, choice, block),
            StatementKind::Assembly(assembly) => self.assembly(body, assembly, block).map(|()| false),
            StatementKind::Broken => Err(Reported),
        }
    }

    fn let_statement(&mut self, body: &mut Body, binding: &syntax::Let, block: &mut checked::Block) -> Check<()> {
        if body.find(&binding.name.text).is_some() || self.constants.contains_key(&binding.name.text) {
            return Err(self.error(Diagnostic::not_designed(
                binding.name.span,
                format!("declaring `{}` again while the first is in scope", binding.name.text),
                NOT_DECIDED,
            )));
        }
        if binding.constant {
            return self.local_constant(body, binding, block);
        }
        let declared = match &binding.written_type {
            Some(written) => self.resolve_type(written).unwrap_or(Type::Unknown),
            None => Type::Unknown,
        };
        let local = body.new_local(declared.machine());
        // A type that holds borrows makes this a stored borrow of what fills its sources (D171).
        let fills = match &binding.written_type {
            Some(written) => match self.let_fills(body, written, &declared) {
                Ok(fills) => fills,
                Err(Reported) => return Err(Reported),
            },
            None => Vec::new(),
        };
        body.expected_fills = if fills.is_empty() { None } else { Some(fills.clone()) };
        self.pointer_borrow = None;
        let result = self.let_value(body, &declared, binding, local, block);
        body.expected_fills = None;
        // A task holds what its call borrows until it's done (D343, D347).
        if let (Ok(()), Some(spawned)) = (&result, self.spawned.take()) {
            if !spawned.borrows.is_empty() {
                body.events.push(crate::flow::Event::Borrow {
                    borrower: local,
                    sources: spawned.borrows,
                    mutable: spawned.mutable,
                    held: false,
                });
            }
            body.tasks.retain(|(each, _)| *each != local);
            body.tasks.push((local, spawned.task));
        }
        // A pointer to a variable keeps it lent for as long as the pointer exists (D315).
        if let (Ok(()), Some((source, path, mutable))) = (&result, self.pointer_borrow.take()) {
            body.events.push(crate::flow::Event::Borrow {
                borrower: local,
                sources: vec![(source, path)],
                mutable,
                held: false,
            });
        }
        if result.is_ok() && !fills.is_empty() {
            let sources: Vec<(checked::LocalId, Vec<usize>)> = fills
                .iter()
                .filter_map(|(_, fill)| body.lookup(fill).map(|variable| (variable.local, Vec::new())))
                .collect();
            body.events.push(crate::flow::Event::Borrow { borrower: local, sources, mutable: false, held: false });
        }
        // The variable exists even if its value is refused, and reading it then is its own error (standard 11.7).
        let origin = if result.is_ok() { Origin::Let } else { Origin::RefusedLet };
        let state = if result.is_ok() { State::Assigned } else { State::Unassigned };
        if result.is_ok() {
            self.bind_event(body, local, &binding.name.text, binding.name.span, &declared);
        }
        // A stored borrow is only lent, and one lent to change can change what it borrows (D2, D174).
        let lent_to_change = matches!(declared, Type::Borrow { mutable: true, .. });
        if let Type::Borrow { .. } = declared {
            body.lent.push(local);
        }
        body.declare(
            Variable {
                name: binding.name.text.clone(),
                value_type: declared,
                mutable: binding.mutable || lent_to_change,
                origin,
                local,
                declared: binding.name.span,
                given: None,
                known_case: None,
            },
            state,
        );
        result
    }

    /// A `const` in a body: worked out while compiling, and fixed (D83).
    fn local_constant(&mut self, body: &mut Body, binding: &syntax::Let, block: &mut checked::Block) -> Check<()> {
        let declared = match &binding.written_type {
            Some(written) => self.resolve_type(written).unwrap_or(Type::Unknown),
            None => Type::Unknown,
        };
        let local = body.new_local(declared.machine());
        let result =
            if declared.is_unknown() { Err(Reported) } else { self.constant_value(body, &binding.value, &declared) };
        let known = result.is_ok();
        let outcome = result.as_ref().map(|_| ()).map_err(|_| Reported);
        if let Ok(value) = result {
            body.known.insert(local, value.clone());
            block.push(checked::Statement::Let { local, value: value.expression(declared.machine()) });
        }
        body.declare(
            Variable {
                name: binding.name.text.clone(),
                value_type: declared,
                mutable: false,
                origin: if known { Origin::Constant } else { Origin::RefusedLet },
                local,
                declared: binding.name.span,
                given: None,
                known_case: None,
            },
            if known { State::Assigned } else { State::Unassigned },
        );
        outcome
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
        if let Type::Borrow { .. } = declared {
            return self.let_borrow(body, binding, local, declared, block);
        }
        if let Type::List { view, .. } = declared
            && *view != super::types::ListView::Owned
        {
            // trace: D367, D379
            if let ExpressionKind::Variable(name) = &binding.value.kind {
                return Err(self.error(Diagnostic::refused(
                    binding.value.span,
                    format!(
                        "`{}` is a view of a list, so storing or returning it writes where it's from, as `Lend from {} {declared}`",
                        name.text, name.text
                    ),
                    "a list view follows a `str` view's borrow rules: passing one needs nothing, and storing or returning one writes its source (D367, D379)",
                )));
            }
            if !matches!(binding.value.kind, ExpressionKind::Empty) {
                return Err(self.not_yet(binding.value.span, "a list view held in a variable".to_string()));
            }
        }
        let value = self.value_into(body, &binding.value, declared)?;
        block.push(checked::Statement::Let { local, value });
        Ok(())
    }

    /// `let $view: Lend from $r reading = Lend $r;`: a stored borrow, which from here on is `$r` in place (D2, D169,
    /// D174).
    fn let_borrow(
        &mut self,
        body: &mut Body,
        binding: &syntax::Let,
        local: checked::LocalId,
        declared: &Type,
        block: &mut checked::Block,
    ) -> Check<()> {
        use syntax::Ownership;
        let Type::Borrow { mutable, held, sources, inner } = declared else { return Err(Reported) };
        let value = &binding.value;
        let wants = if *mutable { Ownership::LendMut } else { Ownership::Lend };
        if let ExpressionKind::Call { .. } = &value.kind {
            body.borrow_call = true;
            let typed = self.expression(body, value, None);
            body.borrow_call = false;
            let typed = typed?;
            let Some((names, locals)) = body.call_borrows.take() else { return Err(Reported) };
            let Type::Borrow { mutable: gives_mut, inner: gives, .. } = &typed.value_type else {
                return Err(self.not_yet(value.span, "a stored borrow from this call".to_string()));
            };
            if gives != inner {
                return Err(self.error(Diagnostic::refused(
                    value.span,
                    format!("this is `{gives}`, where `{inner}` is needed"),
                    super::expressions::CONVERT_WHY,
                )));
            }
            if gives_mut != mutable {
                return Err(self.error(Diagnostic::refused(
                    value.span,
                    format!("this borrow is made with `{}`", wants.word()),
                    "the call and the parameter both say it, and they must match (D2)",
                )));
            }
            if !names.iter().all(|name| sources.contains(name)) {
                return Err(self.error(Diagnostic::refused(
                    value.span,
                    format!("this borrows `{}`, so the type names that", names.join(", ")),
                    "at the call, the result's sources become the arguments (D170)",
                )));
            }
            if sources.iter().any(|source| !names.contains(source)) {
                return Err(self.error(Diagnostic::not_designed(
                    value.span,
                    "a stored borrow that names more than it borrows",
                    NOT_DECIDED,
                )));
            }
            body.events.push(crate::flow::Event::Borrow {
                borrower: local,
                sources: locals.into_iter().map(|source| (source, Vec::new())).collect(),
                mutable: *mutable,
                held: *held,
            });
            block.push(checked::Statement::AliasValue { local, value: typed.value });
            return Ok(());
        }
        let ExpressionKind::Owned { ownership, value: lent, .. } = &value.kind else {
            return Err(self.not_yet(value.span, "a stored borrow from this".to_string()));
        };
        if *ownership != wants {
            return Err(self.error(Diagnostic::refused(
                value.span,
                format!("this borrow is made with `{}`", wants.word()),
                "the call and the parameter both say it, and they must match (D2)",
            )));
        }
        if sources.iter().any(|source| source == "program") {
            return Err(self.error(Diagnostic::not_designed(
                value.span,
                "what can be lent `from program`",
                "not decided yet (standard 12.8)",
            )));
        }
        // A borrow of an item borrows all of its list, and its place goes through the item (D172).
        let mut item_place = None;
        let found = match self.lent_place(body, lent) {
            Some(found) => Some(found),
            None if super::expressions::is_item(lent) => match self.reach(body, lent)? {
                Some(reached) => {
                    let local = reached.root.local;
                    item_place = Some(checked::Place { local, steps: reached.steps });
                    Some((local, Vec::new()))
                }
                None => None,
            },
            None => None,
        };
        let (Some((root_local, path)), Some(root)) = (found, root_name(lent)) else {
            return Err(self.not_yet(value.span, "a stored borrow of this".to_string()));
        };
        let Some(root_variable) = body.lookup(&root.text).cloned() else { return Err(Reported) };
        // Through a stored borrow, the new one names what that one borrows (D174).
        let (originals, through) = match &root_variable.value_type {
            Type::Borrow { mutable: lends_mut, sources: theirs, .. } => {
                if *mutable && !lends_mut {
                    return Err(self.error(Diagnostic::refused(
                        root.span,
                        format!("`{}` only reads what it borrows, so it can't lend it with `Lend.mut`", root.text),
                        "a stored `Lend` can be lent on with `Lend` only (D174, D2)",
                    )));
                }
                (theirs.clone(), Some(root_variable.local))
            }
            _ => (vec![root.text.clone()], None),
        };
        if *mutable && !root_variable.mutable {
            return Err(self.error(Diagnostic::refused(
                root.span,
                format!("`{}` can't change, so it can't be lent with `Lend.mut`", root.text),
                "`let` declares a variable that can't change, and `let mut` one that can (D3)",
            )));
        }
        if through.is_some() && sources.contains(&root.text) {
            let original = originals.join(", ");
            return Err(self.error(Diagnostic::refused(
                value.span,
                format!("this borrows `{original}` through `{}`, so the type names `{original}`", root.text),
                "a borrow taken through a stored borrow names the original value (D174)",
            )));
        }
        if !originals.iter().all(|original| sources.contains(original)) {
            let named = match sources.as_slice() {
                [one] => one.clone(),
                _ => format!("<{}>", sources.join(", ")),
            };
            return Err(self.error(Diagnostic::refused(
                value.span,
                format!("this borrow says it's from `{named}`"),
                "a stored borrow's type says what it borrows (D2)",
            )));
        }
        let typed = self.expression(body, lent, Some(inner))?;
        // A view borrows any string, owned or not (D327).
        let viewed = **inner == Type::Str(super::types::StrSize::View) && matches!(typed.value_type, Type::Str(_));
        if typed.value_type != **inner && !typed.value_type.is_unknown() && !viewed {
            return Err(self.error(Diagnostic::refused(
                lent.span,
                format!("this is `{}`, where `{inner}` is needed", typed.value_type),
                super::expressions::CONVERT_WHY,
            )));
        }
        body.take_read(root_variable.local);
        let site = body.site(root.span);
        let spot = crate::flow::Spot { variable: root.span, place: lent.span, site, marker: site };
        let place = crate::flow::Place { local: root_variable.local, path: path.clone(), shown: String::new() };
        body.events.push(if *mutable {
            crate::flow::Event::LendMut { place, spot }
        } else {
            crate::flow::Event::Read { place, spot }
        });
        let mut source_locals: Vec<(checked::LocalId, Vec<usize>)> = originals
            .iter()
            .filter_map(|original| body.lookup(original).map(|variable| variable.local))
            .map(|source| (source, if through.is_none() { path.clone() } else { Vec::new() }))
            .collect();
        source_locals.extend(through.map(|local| (local, Vec::new())));
        body.events.push(crate::flow::Event::Borrow {
            borrower: local,
            sources: source_locals,
            mutable: *mutable,
            held: *held,
        });
        match item_place {
            Some(place) => block.push(checked::Statement::AliasValue {
                local,
                value: checked::Expression { kind: Kind::PlaceAddress(place), machine_type: MachineType::Address },
            }),
            None => block.push(checked::Statement::Alias { local, target: root_local, path }),
        }
        Ok(())
    }

    /// Leaving a `for` that's handing out items with a duty, before it's handed out all of them (D180).
    fn leaves_items(&mut self, span: Span, list: &str, given: Span) -> Reported {
        self.error(
            Diagnostic::refused(
                span,
                format!("this leaves before the `for` has handed out every item of `{list}`"),
                "a `must_Give` value is handed off with `Give` exactly once, so every item has to be (D2, D180)",
            )
            .with_because(given, format!("`{list}` was given to the `for` here")),
        )
    }

    /// `return Lend $a;` from a function that hands back a borrow, checked against its claim (D170).
    fn return_borrow(&mut self, body: &mut Body, expression: &syntax::Expression) -> Check<checked::Expression> {
        use syntax::Ownership;
        let Type::Borrow { mutable, sources, inner, .. } = body.returns.clone() else { return Err(Reported) };
        let word = if mutable { "Lend.mut" } else { "Lend" };
        let claim = match sources.as_slice() {
            [one] => one.clone(),
            _ => format!("<{}>", sources.join(", ")),
        };
        if let ExpressionKind::Call { .. } = &expression.kind {
            body.borrow_call = true;
            let typed = self.expression(body, expression, None);
            body.borrow_call = false;
            let typed = typed?;
            let Some((names, _)) = body.call_borrows.take() else { return Err(Reported) };
            if !names.iter().all(|name| sources.contains(name)) {
                return Err(self.error(Diagnostic::refused(
                    expression.span,
                    format!("this borrows `{}`, and the function says it's from `{claim}`", names.join(", ")),
                    "the body is checked against the claim (D170)",
                )));
            }
            return Ok(typed.value);
        }
        let ExpressionKind::Owned {
            ownership: ownership @ (Ownership::Lend | Ownership::LendMut), value: lent, ..
        } = &expression.kind
        else {
            return Err(self.error(Diagnostic::refused(
                expression.span,
                format!("this function hands back a borrow, so this says `{word}`"),
                "a returned borrow is made with `Lend` or `Lend.mut`, or comes from a call that hands one back (D170)",
            )));
        };
        if (*ownership == Ownership::LendMut) != mutable {
            return Err(self.error(Diagnostic::refused(
                expression.span,
                format!("this function hands back `{word}`, so this says it"),
                "the claim and what's returned say the same borrow (D170, D2)",
            )));
        }
        let (Some((root_local, path)), Some(root)) = (self.lent_place(body, lent), root_name(lent)) else {
            return Err(self.not_yet(expression.span, "handing back a borrow of this".to_string()));
        };
        let Some(variable) = body.lookup(&root.text).cloned() else { return Err(Reported) };
        let originals = match &variable.value_type {
            Type::Borrow { sources: theirs, .. } => theirs.clone(),
            _ if variable.origin == Origin::Parameter => vec![root.text.clone()],
            _ => {
                return Err(self.error(Diagnostic::refused(
                    expression.span,
                    format!("`{}` is a local, which ends when the function returns", root.text),
                    "a returned borrow comes only from a parameter, a named source or `program` (D170)",
                )));
            }
        };
        if !originals.iter().all(|original| sources.contains(original)) {
            return Err(self.error(Diagnostic::refused(
                expression.span,
                format!("this borrows `{}`, and the function says it's from `{claim}`", originals.join(", ")),
                "the body is checked against the claim (D170)",
            )));
        }
        let typed = self.expression(body, lent, Some(&inner))?;
        if typed.value_type != *inner && !typed.value_type.is_unknown() {
            return Err(self.error(Diagnostic::refused(
                lent.span,
                format!("this is `{}`, where `{inner}` is needed", typed.value_type),
                super::expressions::CONVERT_WHY,
            )));
        }
        body.take_read(variable.local);
        let site = body.site(root.span);
        let spot = crate::flow::Spot { variable: root.span, place: lent.span, site, marker: site };
        let place = crate::flow::Place { local: variable.local, path: path.clone(), shown: String::new() };
        body.events.push(if mutable {
            crate::flow::Event::LendMut { place, spot }
        } else {
            crate::flow::Event::Read { place, spot }
        });
        Ok(checked::Expression {
            kind: Kind::AddressOf { local: root_local, path },
            machine_type: MachineType::Address,
        })
    }

    /// The place a `Lend` names: a variable, or a field inside one.
    fn lent_place(&mut self, body: &Body, lent: &syntax::Expression) -> Option<(checked::LocalId, Vec<usize>)> {
        super::expressions::place_of(body, lent)
            .or_else(|| self.field_place(body, lent).map(|(local, path, _)| (local, path)))
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
        self.bind_event(body, variable.local, &name.text, variable.declared, &variable.value_type);
        Ok(())
    }

    /// `shield sensor: reason "…" … end shield sensor`: a named block with a written reason, whose code runs as it
    /// stands (D34).
    fn shield_statement(
        &mut self,
        body: &mut Body,
        shield: &syntax::Shield,
        block: &mut checked::Block,
    ) -> Check<bool> {
        if body.purity == super::Purity::Pure {
            // trace: D86
            // What's inside is still read, so a `return` there still counts.
            self.error(Diagnostic::refused(
                shield.name.span,
                "a pure function can't hold a shield",
                "a pure function refuses shields and C calls (D86)",
            ));
        }
        self.check_closer(&shield.end, &format!("end shield {}", shield.name.text), super::CLOSER_WHY);
        if shield.reason.is_none() {
            self.error(Diagnostic::refused(
                shield.name.span,
                format!("the shield `{}` has no reason", shield.name.text),
                "a shield is a named block with a written reason (D34)",
            ));
        }
        if let Some(first) = shield.body.first()
            && matches!(first.kind, StatementKind::Requires { .. } | StatementKind::Ensures { .. })
        {
            let keyword = self.slice(first.span).split_whitespace().next().unwrap_or("").len();
            let span = Span::new(self.file, first.span.start as usize, first.span.start as usize + keyword);
            return Err(self.error(Diagnostic::not_designed(
                span,
                "how a shield's own entry and exit are written",
                "not decided yet (standard 12.9)",
            )));
        }
        self.shields += 1;
        let checked = self.block(body, &shield.body);
        self.shields -= 1;
        block.extend(checked.block);
        Ok(checked.leaves)
    }

    /// `on.target aarch64: … else: … end on.target`: both branches are checked, and only the one for the chip being
    /// built is built. What both branches declare outlives the choice, since only one of them is ever there (D233).
    fn on_target(&mut self, body: &mut Body, choice: &syntax::OnTarget, block: &mut checked::Block) -> Check<bool> {
        self.check_closer(&choice.end, "end on.target", super::CLOSER_WHY);
        let chip = choice.chip.text.clone();
        if !matches!(chip.as_str(), "aarch64" | "x86_64") {
            return Err(self.error(Diagnostic::not_designed(
                choice.chip.span,
                format!("the chip `{chip}`"),
                "not decided yet (standard 12.5)",
            )));
        }
        let active = self.target.chip() == chip;
        self.chips.push(Some(chip));
        let (then, then_declared) = self.branch(body, &choice.then);
        self.chips.pop();
        let otherwise = choice.otherwise.as_ref().map(|statements| {
            self.chips.push(None);
            let branch = self.branch(body, statements);
            self.chips.pop();
            branch
        });
        let other_declared = otherwise.as_ref().map(|(_, declared)| declared.clone()).unwrap_or_default();
        let (chosen, declared, other) = match (active, otherwise) {
            (true, _) => (Some(then), then_declared, other_declared),
            (false, Some((checked, declared))) => (Some(checked), declared, then_declared),
            (false, None) => (None, Vec::new(), Vec::new()),
        };
        for variable in declared {
            let Some(theirs) = other.iter().find(|each| each.name == variable.name) else { continue };
            if theirs.value_type != variable.value_type {
                return Err(self.not_yet(
                    theirs.declared,
                    format!("`{}` declared with different types in an `on.target`'s branches", variable.name),
                ));
            }
            body.declare(variable, super::State::Assigned);
        }
        let Some(chosen) = chosen else { return Ok(false) };
        block.extend(chosen.block);
        Ok(chosen.leaves)
    }

    /// One branch of an `on.target`, with what it declared, including through assembly blocks' `writes:`.
    fn branch(&mut self, body: &mut Body, statements: &[syntax::Statement]) -> (Checked, Vec<Variable>) {
        let outer = std::mem::take(&mut self.assembly_writes);
        let checked = self.block(body, statements);
        let mut declared = std::mem::replace(&mut self.assembly_writes, outer);
        declared.extend(checked.declared.iter().cloned());
        (checked, declared)
    }

    /// An assembly block: inside a shield, in an `on.target` branch naming its chip; values go into registers, the
    /// raw lines run, and registers come out into new variables (D233).
    fn assembly(&mut self, body: &mut Body, assembly: &syntax::Assembly, block: &mut checked::Block) -> Check<()> {
        const CHIP_WHY: &str = "assembly is for one chip, so it sits in an `on.target` branch that names it, and has no `target:` line (D233)";
        let keyword =
            Span::new(self.file, assembly.opener.start as usize, (assembly.opener.end as usize).saturating_sub(1));
        if self.shields == 0 {
            return Err(self.error(Diagnostic::refused(
                keyword,
                "an assembly block must be inside a shield",
                "a shield is the only place dangerous code may live (D34, D38, D233)",
            )));
        }
        let Some(Some(chip)) = self.chips.last().cloned() else {
            return Err(self.error(Diagnostic::refused(
                keyword,
                "an assembly block goes inside an `on.target` choice naming its chip",
                CHIP_WHY,
            )));
        };
        let mut reads = Vec::new();
        for (register, value) in &assembly.reads {
            if let ExpressionKind::Owned { .. } = &value.kind {
                return Err(self.error(Diagnostic::not_designed(
                    value.span,
                    "lending an address into a register",
                    "not decided yet (standard 12.5)",
                )));
            }
            let typed = self.expression(body, value, None)?;
            let bits = register_bits(&chip, &register.text);
            let fits = matches!(typed.value_type, Type::Integer { kind, .. } if Some(kind.bits) == bits);
            if !fits {
                return Err(self.error(Diagnostic::refused(
                    value.span,
                    format!("this is `{}`, and `{}` holds {} bits", typed.value_type, register.text, bits.unwrap_or(0)),
                    "a value goes into a register of its own width, with `as` first if it's narrower, so nothing widens silently (D233)",
                )));
            }
            reads.push((register.text.clone(), typed.value));
        }
        let mut writes = Vec::new();
        for (register, name) in &assembly.writes {
            let bare = name.text.as_str();
            if body.lookup(bare).is_some() {
                return Err(self.error(Diagnostic::not_designed(
                    name.span,
                    format!("declaring `{bare}` again while the first is in scope"),
                    super::NOT_DECIDED,
                )));
            }
            let Some(bits) = register_bits(&chip, &register.text) else {
                return Err(self.not_yet(register.span, format!("the register `{}`", register.text)));
            };
            let value_type =
                Type::Integer { kind: super::types::IntegerKind { bits, signed: true }, bounds: None, unit: None };
            let local = body.new_local(value_type.machine());
            let variable = Variable {
                name: bare.to_string(),
                value_type: value_type.clone(),
                mutable: false,
                origin: super::Origin::Let,
                local,
                declared: name.span,
                given: None,
                known_case: None,
            };
            body.declare(variable.clone(), super::State::Assigned);
            self.assembly_writes.push(variable);
            writes.push((register.text.clone(), local, value_type.machine()));
        }
        let lines = assembly.lines.iter().map(|span| self.slice(*span).to_string()).collect();
        let clobbers = assembly.clobbers.iter().map(|register| register.text.clone()).collect();
        block.push(checked::Statement::Assembly { lines, reads, writes, clobbers, intel: chip == "x86_64" });
        Ok(())
    }

    /// `finish $f;` ends a `must_Give` duty (D129).
    fn finish_statement(&mut self, body: &mut Body, span: Span, value: &syntax::Expression) -> Check<()> {
        const FINISH_WHY: &str = "`finish` ends a `must_Give` duty (D129)";
        let ExpressionKind::Variable(name) = &value.kind else {
            return Err(self.error(Diagnostic::refused(
                value.span,
                "`finish` goes on a variable that holds the duty",
                FINISH_WHY,
            )));
        };
        let Some(variable) = body.lookup(&name.text).cloned() else {
            return Err(self.error(Diagnostic::refused(
                name.span,
                format!("`{}` isn't declared", name.text),
                FINISH_WHY,
            )));
        };
        if !self.must_give(&variable.value_type) {
            return Err(self.error(Diagnostic::refused(
                name.span,
                format!("`{}` is `{}`, which has no duty to end", name.text, variable.value_type),
                FINISH_WHY,
            )));
        }
        if body.lent.contains(&variable.local) {
            return Err(self.error(Diagnostic::refused(
                name.span,
                format!("`{}` is only lent here, so its duty isn't this code's to end", name.text),
                "`Lend` is temporary access, and the caller keeps it (D2, D129)",
            )));
        }
        // The `must_Give` fields inside have to be moved out first (D177, D185).
        let mut duties = Vec::new();
        if let Type::Struct { id, .. } = variable.value_type.read_as()
            && let Some(info) = self.aggregates.structs.get(*id).cloned()
        {
            for (index, field) in info.fields.iter().enumerate() {
                if self.must_give(&field.value_type) {
                    duties.push(crate::flow::Place {
                        local: variable.local,
                        path: vec![index],
                        shown: format!("{}.{}", name.text, field.name),
                    });
                }
            }
        }
        body.events.push(crate::flow::Event::Finish {
            local: variable.local,
            spot: crate::flow::Spot { variable: name.span, place: name.span, site: span, marker: span },
            duties,
        });
        Ok(())
    }

    /// `change $x = …;` (D3, D125).
    fn change(
        &mut self,
        body: &mut Body,
        span: Span,
        target: &syntax::Expression,
        value: &syntax::Expression,
        block: &mut checked::Block,
    ) -> Check<()> {
        if has_item(target) {
            return self.change_item(body, span, target, value, block);
        }
        if let ExpressionKind::Field { .. } = &target.kind {
            return self.change_field(body, target, value, block);
        }
        let ExpressionKind::Variable(name) = &target.kind else {
            return Err(self.not_yet(target.span, "`change` on this".to_string()));
        };
        let Some(index) = body.find(&name.text) else {
            if self.constants.contains_key(&name.text) {
                return Err(self.error(Diagnostic::refused(
                    name.span,
                    format!("`{}` is a constant, so it can't change", name.text),
                    "a constant's value is fixed at compile time (D83)",
                )));
            }
            return Err(self.error(Diagnostic::refused(
                name.span,
                format!("`{}` isn't declared", name.text),
                "`let` declares a variable (D2)",
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
        if variable.origin == Origin::Constant {
            return Err(self.error(Diagnostic::refused(
                name.span,
                format!("`{}` is a constant, so it can't change", name.text),
                "a constant's value is fixed at compile time (D83)",
            )));
        }
        if !variable.mutable {
            if body.chunk_copies.contains(&name.text) {
                // trace: D303
                return Err(self.error(Diagnostic::refused(
                    name.span,
                    format!("`{}` is a copy from `simd_chunks`, so changing it is refused", name.text),
                    "`simd_chunks` only reads, and `.%%simd_chunks.mut` writes each chunk back into the list or array (D303, D358)",
                )));
            }
            if variable.origin == Origin::Parameter && variable.value_type.is_copied() {
                return Err(self.error(Diagnostic::refused(
                    name.span,
                    format!("`{}` is a plain parameter, so it can't change", name.text),
                    "a function can't quietly change its own copy, and `Lent.mut` says it changes the caller's (D184, D125)",
                )));
            }
            return Err(self.error(Diagnostic::refused(
                name.span,
                format!("`{}` can't change", name.text),
                "`let` declares a variable that can't change, and `let mut` one that can (D3)",
            )));
        }
        self.not_while_reading(body, &name.text, name.span)?;
        let value = self.value_into(body, value, variable.value_type.read_as())?;
        block.push(checked::Statement::Set { local: variable.local, value });
        body.states[index] = State::Assigned;
        // A number changing matters to a pointer that borrows it (D315).
        if !variable.value_type.is_copied() || body.pointed_at.contains(&variable.local) {
            // The whole statement is where a test's trust ended (D104).
            let spot = crate::flow::Spot { variable: name.span, place: name.span, site: name.span, marker: span };
            body.events.push(crate::flow::Event::Change {
                place: crate::flow::Place { local: variable.local, path: Vec::new(), shown: String::new() },
                spot,
                written: name.span,
            });
        }
        Ok(())
    }

    /// `change $r.celsius = 22;`: one field, if the whole variable can change (D125).
    fn change_field(
        &mut self,
        body: &mut Body,
        target: &syntax::Expression,
        value: &syntax::Expression,
        block: &mut checked::Block,
    ) -> Check<()> {
        // A `for` reading this field in place, or reading the whole variable, stops it changing (D2, D125).
        if let Some(root) = super::expressions::root_name(target) {
            let written = self.slice(target.span).to_string();
            if body.reading_parts.iter().any(|each| *each == written) || body.reading.contains(&root.text) {
                return Err(self.error(Diagnostic::refused(
                    target.span,
                    format!("a `for` is reading `{}` in place here", root.text),
                    "`Lend.mut` means nothing else uses it meanwhile, and `for` is using it (D2)",
                )));
            }
            if body.reading_parts.iter().any(|each| part_root(each) == root.text) {
                return Err(self.error(Diagnostic::not_designed(
                    target.span,
                    format!("changing one part of `{}` while a `for` reads another", root.text),
                    NOT_DECIDED,
                )));
            }
        }
        // Walk down to the variable, then back up through the fields.
        let mut names = Vec::new();
        let mut at = target;
        while let ExpressionKind::Field { base, field } = &at.kind {
            names.push(field);
            at = base;
        }
        names.reverse();
        let ExpressionKind::Variable(root) = &at.kind else {
            return Err(self.error(Diagnostic::refused(
                target.span,
                "`change` works on a variable, or a field of one",
                "`change` reassigns a variable or one field of it (D3, D125)",
            )));
        };
        let Some(variable) = body.lookup(&root.text).cloned() else {
            if self.constants.contains_key(&root.text) {
                return Err(self.error(Diagnostic::refused(
                    target.span,
                    format!("`{}` is a constant, so it can't change", root.text),
                    "a constant's value is fixed at compile time (D83)",
                )));
            }
            return Err(self.error(Diagnostic::refused(
                root.span,
                format!("`{}` isn't declared", root.text),
                "`let` declares a variable (D2)",
            )));
        };
        let mut path = Vec::new();
        let mut current = variable.value_type.read_as().clone();
        for field in names {
            let reached = at.span.to(field.span);
            if let Type::Union { id, name } = &current
                && let Some(holding) = self.aggregates.unions[*id]
                    .cases
                    .iter()
                    .position(|case| case.fields.iter().any(|each| each.name == field.text))
            {
                let shown = self.slice(at.span).to_string();
                if variable.known_case.is_some() {
                    return Err(self.error(Diagnostic::not_designed(
                        reached,
                        "`change` on a value inside a union's case, and whether the test's trust survives it",
                        NOT_DECIDED,
                    )));
                }
                let case = self.aggregates.unions[*id].cases[holding].name.clone();
                return Err(self.error(Diagnostic::refused(
                    reached,
                    format!("`{shown}` isn't known to be `{name}.{case}` here"),
                    format!("inside `if {shown}: {name}.{case}:` the compiler trusts it, and outside that's a compile error (D9)"),
                )));
            }
            if matches!(current, Type::Failure { .. }) && !variable.mutable {
                return Err(self.error(Diagnostic::refused(
                    reached,
                    format!("`{}` can't change, so neither can its field", root.text),
                    "a field or item can be changed if the whole variable is `let mut`, or the parameter `Lent.mut` (D125, D133)",
                )));
            }
            let Type::Struct { id, name } = &current else {
                return Err(self.not_yet(field.span, "changing a field of this".to_string()));
            };
            let info = self.aggregates.structs[*id].clone();
            let Some(index) = info.fields.iter().position(|each| each.name == field.text) else {
                return Err(self.error(Diagnostic::refused(
                    field.span,
                    format!("`{name}` has no field `{}`", field.text),
                    "a field read is a plain `.` (D57)",
                )));
            };
            path.push(index);
            current = info.fields[index].value_type.clone();
        }
        if !variable.mutable {
            return Err(self.error(Diagnostic::refused(
                target.span,
                format!("`{}` can't change, so neither can its field", root.text),
                "a field or item can be changed if the whole variable is `let mut`, or the parameter `Lent.mut` (D125, D133)",
            )));
        }
        let value = self.value_into(body, value, &current)?;
        let spot = crate::flow::Spot { variable: root.span, place: target.span, site: root.span, marker: target.span };
        body.events.push(crate::flow::Event::Change {
            place: crate::flow::Place { local: variable.local, path: path.clone(), shown: String::new() },
            spot,
            written: target.span,
        });
        block.push(checked::Statement::SetField { local: variable.local, path, value });
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
                Ok(checked::Statement::Return(body.result.map(|_| self.succeed(body, None))))
            }
            Some(expression) if matches!(returns, Type::Borrow { .. }) => {
                self.return_borrow(body, expression).map(|value| checked::Statement::Return(Some(value)))
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
                body.expected_fills.clone_from(&body.return_fills);
                let value = self.value_into(body, expression, &returns);
                body.expected_fills = None;
                match value {
                    Ok(value) if !body.top.ensures.is_empty() => {
                        // `ensures` reads the value being returned as `result` (D29).
                        let local = body.new_local(returns.machine());
                        block.push(checked::Statement::Let { local, value });
                        let top = body.top.clone();
                        let checks = self.ensures_checks(body, &top, local, statement.span);
                        block.extend(checks);
                        let value = checked::Expression { kind: Kind::Local(local), machine_type: returns.machine() };
                        let value = if body.result.is_some() { self.succeed(body, Some(value)) } else { value };
                        Ok(checked::Statement::Return(Some(value)))
                    }
                    Ok(value) => {
                        let value = if body.result.is_some() { self.succeed(body, Some(value)) } else { value };
                        Ok(checked::Statement::Return(Some(value)))
                    }
                    Err(Reported) => Err(Reported),
                }
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
                // A writer lent in is the caller's to finish (D273).
                if matches!(variable.value_type, Type::Writer(writer) if writer.buffered)
                    && variable.given.is_none()
                    && !body.lent.contains(&variable.local)
                    && variable.origin != Origin::RefusedLet
                {
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

    /// `$xs.std::%%append(…)` as a statement, alone, with `?? void`, or with `otherwise:` arms for running out of
    /// memory (D24, D96, D321, D324). `None` for anything else.
    fn append_statement(
        &mut self,
        body: &mut Body,
        expression: &syntax::Expression,
        block: &mut checked::Block,
    ) -> Option<Check<()>> {
        use super::memory::Handling;
        let (call, handling) = match &expression.kind {
            ExpressionKind::Instead { value, fallback, .. } if matches!(&fallback.kind, ExpressionKind::Path(path) if path.is_single("void")) => {
                (value.as_ref(), Handling::Ignore)
            }
            ExpressionKind::Otherwise { value, handling: syntax::Handling::Arms { arms, .. }, .. } => {
                (value.as_ref(), Handling::Arms(arms))
            }
            _ => (expression, Handling::None),
        };
        let ExpressionKind::MethodCall { receiver, method, arguments, .. } = &call.kind else { return None };
        let is_append = method.text() == "std::%%append"
            || (self.method_uses.get(&method.text()).is_some_and(|full| full == "std::coll::list.%%append")
                && matches!(self.expression_type_quietly(body, receiver), Some(Type::List { .. })));
        if !is_append {
            return None;
        }
        let allocates =
            arguments.iter().any(|argument| argument.label.as_ref().is_some_and(|label| label.text == "alloc"));
        if body.purity == super::Purity::Pure && allocates {
            return Some(Err(self.error(Diagnostic::not_designed(
                call.span,
                "whether a `fn.pure` may allocate",
                "not decided yet (standard 12.5)",
            ))));
        }
        Some((|| {
            let (push, flag) = self.append(body, expression, receiver, arguments, &handling, block)?;
            block.push(push);
            if let (Handling::Arms(arms), Some(flag)) = (&handling, flag) {
                let mut then = Vec::new();
                let before = body.states.clone();
                for arm in *arms {
                    let name = arm.pattern.text();
                    if name != "out_of_memory" {
                        return Err(self.error(Diagnostic::refused(
                            arm.pattern.span,
                            format!("`append` fails only with `out_of_memory`, not `{name}`"),
                            "a list that grows fails with `out_of_memory` when its allocator has no room (D321, D324)",
                        )));
                    }
                    let checked = self.block(body, &arm.body);
                    then.extend(checked.block);
                }
                body.states = Body::join_states(&before, &body.states);
                block.push(checked::Statement::If {
                    condition: checked::Expression {
                        kind: checked::ExpressionKind::Local(flag),
                        machine_type: MachineType::Bool,
                    },
                    then,
                    otherwise: Vec::new(),
                });
            }
            Ok(())
        })())
    }

    fn expression_statement(
        &mut self,
        body: &mut Body,
        expression: &syntax::Expression,
        block: &mut checked::Block,
    ) -> Check<()> {
        // A print can fail with `print_failed` (D96), which this compiler's writers never do, so its handling is
        // checked and the print runs as it is.
        let inner = match &expression.kind {
            ExpressionKind::Pass { value, .. }
            | ExpressionKind::Instead { value, .. }
            | ExpressionKind::Otherwise { value, .. } => Some(value.as_ref()),
            _ => None,
        };
        if let Some(inner) = inner
            && self.is_library_print(inner)
        {
            if let ExpressionKind::Pass { .. } = &expression.kind {
                if body.is_main {
                    return Err(self.error(Diagnostic::not_designed(
                        expression.span,
                        "`?` in `fn.main`",
                        "not decided yet (standard 12.6)",
                    )));
                }
                if !body.failures.iter().any(|failure| failure.short == "print_failed") {
                    return Err(self.error(Diagnostic::refused(
                        expression.span,
                        "this passes up `print_failed`, which this function doesn't list",
                        "`?` is a compile error if the function doesn't list what it passes up (D47)",
                    )));
                }
            }
            // The handler is still checked for its `*` (D46), even though a print here never fails.
            // Its arms are checked where they're used too, since a `break` in one needs a loop here (D128). They
            // never run, so what they build is dropped.
            let arms = match &expression.kind {
                ExpressionKind::Otherwise { handling: syntax::Handling::Handler(path), .. } => {
                    self.handler_arms(path)?
                }
                ExpressionKind::Otherwise { handling: syntax::Handling::Arms { arms, .. }, .. } => arms.clone(),
                _ => Vec::new(),
            };
            let before = body.states.clone();
            let start = body.states.len();
            for arm in &arms {
                let result = body.new_local(MachineType::Void);
                body.handlers.push(super::failures::Handler {
                    value_type: Type::Void,
                    result,
                    loops: body.loops.len(),
                    variables: body.variables.len(),
                });
                self.block(body, &arm.body);
                body.handlers.pop();
                body.forget_after(start);
                body.states.clone_from(&before);
            }
            if let Some(statement) = self.library_statement(body, inner)? {
                block.push(statement);
                return Ok(());
            }
        }
        if let Some(statement) = self.memory_statement(body, expression) {
            block.push(statement?);
            return Ok(());
        }
        if let Some(statement) = self.library_statement(body, expression)? {
            block.push(statement);
            return Ok(());
        }
        if let Some(done) = self.append_statement(body, expression, block) {
            return done;
        }
        if let Some(done) = self.compare_swap_statement(body, expression, block) {
            return done;
        }
        if !body.handlers.is_empty()
            && matches!(
                expression.kind,
                ExpressionKind::Text(_)
                    | ExpressionKind::Integer(_)
                    | ExpressionKind::Float(_)
                    | ExpressionKind::Variable(_)
                    | ExpressionKind::Bool(_)
            )
        {
            let shown = self.slice(expression.span).to_string();
            return Err(self.error(Diagnostic::refused_fix(
                expression.span,
                format!("instead {shown}"),
                "`instead` gives the value used in place of a failed call (D139)",
            )));
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
        for (index, branch) in chain.branches.iter().enumerate() {
            let mark = body.events.len();
            let condition = self.branch_condition(body, &branch.condition);
            // A later test only runs once the ones before it were false, so what it does goes in that `else`.
            let condition_events =
                if index == 0 { Vec::new() } else { body.events.split_off(mark.min(body.events.len())) };
            let before = body.states.clone();
            body.scopes.push(Vec::new());
            if let Ok((_, Some(narrowed))) = &condition {
                body.declare(narrowed.clone(), State::Assigned);
            }
            // A refused name on a test still exists inside, so reading it isn't a second error.
            if let (Err(_), Condition::Test { binding: Some(binding), .. }) = (&condition, &branch.condition) {
                let local = body.new_local(Type::Unknown.machine());
                body.declare(
                    Variable {
                        name: binding.text.clone(),
                        value_type: Type::Unknown,
                        mutable: false,
                        origin: Origin::Let,
                        local,
                        declared: binding.span,
                        given: None,
                        known_case: None,
                    },
                    State::Assigned,
                );
            }
            let mut checked = self.block(body, &branch.body);
            // A test on a union's case is trusted inside, until a `change` or `Lend.mut` (D9, D104).
            if let Ok((_, Some(narrowed))) = &condition
                && narrowed.known_case.is_some()
                && let Type::Union { id, name } = &narrowed.value_type
                && let Some(case) = narrowed.known_case.and_then(|case| self.aggregates.unions[*id].cases.get(case))
            {
                let what = format!("{name}.{}", case.name);
                checked.block.insert(
                    0,
                    checked::Statement::Flow(vec![crate::flow::Event::Trust { local: narrowed.local, what }]),
                );
            }
            body.scopes.pop();
            body.forget_after(start);
            if !checked.leaves {
                ends.push(body.states[..start].to_vec());
            }
            body.states.clone_from(&before);
            branches.push((condition.map(|(value, _)| value), checked.block, condition_events));
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
        for (condition, then, events) in branches.into_iter().rev() {
            match condition {
                Ok(condition) => {
                    built = vec![checked::Statement::If { condition, then, otherwise: built }];
                    if !events.is_empty() {
                        built.insert(0, checked::Statement::Flow(events));
                    }
                }
                Err(Reported) => failed = true,
            }
        }
        if failed {
            return Err(Reported);
        }
        block.extend(built);
        Ok(all_leave)
    }

    /// `match`: a union's cases, every one covered (D9, D33, D139).
    fn match_statement(
        &mut self,
        body: &mut Body,
        span: Span,
        choice: &syntax::Match,
        block: &mut checked::Block,
    ) -> Check<bool> {
        self.check_closer(&choice.end, "end match", super::CLOSER_WHY);
        let typed = self.expression(body, &choice.subject, None)?;
        let Type::Union { id, name } = typed.value_type.clone() else {
            if typed.value_type.is_unknown() {
                return Err(Reported);
            }
            return Err(self.not_yet(choice.subject.span, format!("`match` on `{}`", typed.value_type)));
        };
        let union = self.aggregates.unions[id].clone();
        let local = body.new_local(typed.value.machine_type.clone());
        block.push(checked::Statement::Let { local, value: typed.value });
        let start = body.states.len();
        let mut covered = vec![false; union.cases.len()];
        let mut arms = Vec::new();
        let mut ends = Vec::new();
        let mut failed = false;
        for arm in &choice.arms {
            let written = self.union_case_written(&arm.pattern);
            let case_name = match written.strip_prefix(&format!("{name}.")) {
                Some(case_name) => case_name.to_string(),
                None if union.cases.iter().any(|case| case.name == written) => {
                    self.error(Diagnostic::refused_fix(
                        arm.pattern.span,
                        format!("{name}.{written}"),
                        "a union's cases are narrowings, such as `status.failed` (D9, D33)",
                    ));
                    failed = true;
                    continue;
                }
                None => {
                    self.not_yet(arm.pattern.span, format!("the case `{written}`"));
                    failed = true;
                    continue;
                }
            };
            let Some(case) = union.cases.iter().position(|each| each.name == case_name) else {
                self.error(Diagnostic::refused(
                    arm.pattern.span,
                    format!("`{name}` has no case `{case_name}`"),
                    "a union's cases are narrowings (D33)",
                ));
                failed = true;
                continue;
            };
            covered[case] = true;
            let before = body.states.clone();
            body.scopes.push(Vec::new());
            if arm.binding.is_some() && !body.lent.contains(&local) {
                body.lent.push(local);
            }
            if let Some(binding) = &arm.binding {
                body.declare(
                    Variable {
                        name: binding.text.clone(),
                        value_type: Type::Union { id, name: name.clone() },
                        mutable: false,
                        origin: Origin::Let,
                        local,
                        declared: binding.span,
                        given: None,
                        known_case: Some(case),
                    },
                    State::Assigned,
                );
            }
            let subject = match &choice.subject.kind {
                ExpressionKind::Variable(variable) => Some(variable.text.clone()),
                _ => None,
            };
            body.matching.extend(subject.clone());
            let mut checked = self.block(body, &arm.body);
            // The name a case gives is a borrow of what's matched, so it ends if that's changed (D139, D169).
            if let Some(binding) = &arm.binding
                && let ExpressionKind::Variable(subject_name) = &choice.subject.kind
                && let Some(source) = body.lookup(&subject_name.text).map(|variable| variable.local)
            {
                checked.block.insert(
                    0,
                    checked::Statement::Flow(vec![
                        crate::flow::Event::Bind {
                            local,
                            name: binding.text.clone(),
                            declared: binding.span,
                            must_give: false,
                            origin: crate::flow::Origin::Lent,
                            why: None,
                        },
                        crate::flow::Event::Borrow {
                            borrower: local,
                            sources: vec![(source, Vec::new())],
                            mutable: false,
                            held: false,
                        },
                    ]),
                );
            }
            if subject.is_some() {
                body.matching.pop();
            }
            body.scopes.pop();
            body.forget_after(start);
            if !checked.leaves {
                ends.push(body.states.clone());
            }
            body.states.clone_from(&before);
            arms.push((case, checked.block));
        }
        let missing: Vec<String> = union
            .cases
            .iter()
            .zip(&covered)
            .filter(|(_, done)| !**done)
            .map(|(case, _)| format!("`{name}.{}`", case.name))
            .collect();
        if !missing.is_empty() && !failed {
            let opener = Span::new(self.file, span.start as usize, choice.subject.span.end as usize + 1);
            let list = match missing.as_slice() {
                [one] => one.clone(),
                [rest @ .., last] => format!("{} and {last}", rest.join(", ")),
                [] => String::new(),
            };
            return Err(self.error(Diagnostic::refused(
                opener,
                format!("this `match` misses {list}"),
                "`match` must cover every case (D9)",
            )));
        }
        if failed {
            return Err(Reported);
        }
        let all_leave = ends.is_empty();
        if let Some(first) = ends.first().cloned() {
            body.states = ends.iter().skip(1).fold(first, |joined, each| Body::join_states(&joined, each));
        }
        let tag = checked::Expression {
            kind: Kind::Tag(Box::new(checked::Expression {
                kind: Kind::Local(local),
                machine_type: MachineType::Union(id),
            })),
            machine_type: MachineType::Integer { bits: 32, signed: true },
        };
        let mut built: checked::Block = Vec::new();
        for (case, statements) in arms.into_iter().rev() {
            let number = checked::Expression {
                kind: Kind::Integer(i128::try_from(case).unwrap_or(0)),
                machine_type: MachineType::Integer { bits: 32, signed: true },
            };
            built = vec![checked::Statement::If {
                condition: compare(checked::Comparison::Equal, tag.clone(), number),
                then: statements,
                otherwise: built,
            }];
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
                    return Err(self.error(Diagnostic::not_designed(
                        binding.span,
                        "naming the case in a test, which D139 only shows on `case`",
                        NOT_DECIDED,
                    )));
                }
                let ExpressionKind::Variable(name) = &subject.kind else {
                    return Err(self.not_yet(subject.span, "the test on something that isn't a variable".to_string()));
                };
                let typed = self.expression(body, subject, None)?;
                if let Type::Union { id, name: union_name } = &typed.value_type
                    && let syntax::TypeKind::Named { path, .. } = &written_type.kind
                {
                    let union = self.aggregates.unions[*id].clone();
                    let written = self.union_case_written(path);
                    let Some(case_name) = written.strip_prefix(&format!("{union_name}.")) else {
                        return Err(self.not_yet(written_type.span, format!("the test for `{written}`")));
                    };
                    let Some(case) = union.cases.iter().position(|each| each.name == case_name) else {
                        return Err(self.error(Diagnostic::refused(
                            written_type.span,
                            format!("`{union_name}` has no case `{case_name}`"),
                            "a union's cases are narrowings (D33)",
                        )));
                    };
                    let tag = checked::Expression {
                        kind: Kind::Tag(Box::new(typed.value.clone())),
                        machine_type: MachineType::Integer { bits: 32, signed: true },
                    };
                    let number = checked::Expression {
                        kind: Kind::Integer(i128::try_from(case).unwrap_or(0)),
                        machine_type: MachineType::Integer { bits: 32, signed: true },
                    };
                    let Some(original) = body.lookup(&name.text).cloned() else { return Err(Reported) };
                    let narrowed = Variable { known_case: Some(case), ..original };
                    return Ok((compare(checked::Comparison::Equal, tag, number), Some(narrowed)));
                }
                // A union's case tested on something that isn't that union (D9).
                if let syntax::TypeKind::Named { path, .. } = &written_type.kind
                    && let Some((owner, _)) = path.text().rsplit_once('.')
                    && matches!(self.aggregates.names.get(owner), Some(Type::Union { .. }))
                    && !typed.value_type.is_unknown()
                {
                    // trace: D9
                    return Err(self.error(Diagnostic::refused(
                        written_type.span,
                        format!("`{}` is `{}`, so it has no such case", name.text, typed.value_type),
                        "`if $s: status.failed:` tests one case of a union (D9)",
                    )));
                }
                let tested = self.resolve_type(written_type)?;
                match (&typed.value_type, &tested) {
                    (Type::Integer { kind: from, .. }, Type::Integer { kind: to, bounds: Some(bounds), .. })
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
                        Type::Float { bits: from_bits, kind: from_kind, bounds: None, .. },
                        Type::Float { bits, kind, bounds: Some(bounds), .. },
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
                    (Type::Float { .. } | Type::Integer { .. }, Type::Float { .. } | Type::Integer { .. })
                    | (Type::Bool, Type::Bool)
                    | (Type::Str(_), Type::Str(_)) => Err(self.error(Diagnostic::not_designed(
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
        let mark = body.events.len();
        let condition = match condition {
            Condition::Plain(expression) => self.condition_value(body, expression)?.value,
            Condition::Test { written_type, .. } => {
                return Err(self.error(Diagnostic::not_designed(
                    written_type.span,
                    "the test in a `while`",
                    "not decided yet (standard 12.4)",
                )));
            }
        };
        // The condition runs at the top of every round, so what it does goes inside the loop.
        let condition_events = body.events.split_off(mark.min(body.events.len()));
        let before = body.states.clone();
        let (id, loop_body, breaks) = self.loop_body(body, None, inner);
        // The body may not have run at all, or may have stopped at a `break`.
        let mut joined = Body::join_states(&before, &body.states[..before.len()]);
        for each in breaks {
            joined = Body::join_states(&joined, &each[..before.len()]);
        }
        body.states = joined;
        let mut statements = vec![
            checked::Statement::Flow(condition_events),
            checked::Statement::If {
                condition: not(condition),
                then: vec![checked::Statement::Break(id)],
                otherwise: Vec::new(),
            },
        ];
        statements.extend(loop_body);
        block.push(checked::Statement::Loop { id, body: statements, opener: body.loop_opener });
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
        let variables = body.variables.len();
        let defers = body.defers.len();
        body.loops.push(Loop {
            id,
            label: label.map(|name| name.text.clone()),
            breaks: Some(Vec::new()),
            variables,
            defers,
        });
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
        block.push(checked::Statement::Loop { id, body: loop_body, opener: body.loop_opener });
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
        if let Some((_, list, given)) = body.give_loops.iter().find(|(outside, _, _)| *outside >= target).cloned() {
            return Err(self.leaves_items(span, &list, given));
        }
        // Leaving the loop leaves its variables behind, and their duties with them (D2).
        let start = body.loops[target].variables.min(body.variables.len());
        let locals = body.variables[start..].iter().map(|variable| variable.local).collect();
        body.events.push(crate::flow::Event::Leave { span, locals: Some(locals) });
        if let Some(breaks) = &mut body.loops[target].breaks {
            breaks.push(states);
        }
        block.push(checked::Statement::Break(id));
        Ok(())
    }

    /// `for $chunk in $xs.std::%simd_chunks.pad(lanes: 4, pad: 0):`, `.exact` or `.leftovers`, and
    /// `.%%simd_chunks.mut.…`, which writes each chunk back (D299, D300, D303, D358). `None` for any other loop.
    #[allow(clippy::too_many_lines)]
    fn chunk_loop(
        &mut self,
        body: &mut Body,
        span: Span,
        each: &syntax::For,
        block: &mut checked::Block,
    ) -> Option<Check<()>> {
        const MARK_WHY: &str =
            "`.%` lends a receiver read-only and `.%%` lends it to change, so the call shows which (D211)";
        let ExpressionKind::MethodCall { receiver, method, arguments, .. } = &each.source.kind else { return None };
        let text = method.text();
        if !text.contains("simd_chunks") {
            return None;
        }
        Some((|| {
            let at = |offset: usize, length: usize| {
                let start = method.span.start as usize + offset;
                Span::new(method.span.file, start, start + length)
            };
            let Some(rest) = text.strip_prefix("std::") else {
                // trace: D115
                return Err(self.error(Diagnostic::refused_fix(
                    method.span,
                    format!("std::{text}"),
                    "a method from a kit says its path, because bare means base or your own (D115)",
                )));
            };
            let marks = rest.len() - rest.trim_start_matches('%').len();
            let named = &rest[marks..];
            let name_offset = 5 + marks;
            if let Some(after) = named.strip_prefix("simd_chunks_mut") {
                // trace: D358
                return Err(self.error(Diagnostic::refused_fix(
                    at(name_offset, named.len()),
                    format!("simd_chunks.mut{after}"),
                    "Cyborg narrows with a dot everywhere, as in `ptr.raw.mut` and `Lend.mut`, so it's `simd_chunks.mut` (D358)",
                )));
            }
            let changes = named.starts_with("simd_chunks.mut");
            let base = if changes { "simd_chunks.mut" } else { "simd_chunks" };
            if changes != (marks == 2) {
                let fixed = format!("{}{named}", if changes { "%%" } else { "%" });
                return Err(self.error(Diagnostic::refused_fix(at(5, marks + named.len()), fixed, MARK_WHY)));
            }
            let narrowing = named.strip_prefix(base).and_then(|after| after.strip_prefix('.')).unwrap_or("");
            if !matches!(narrowing, "pad" | "exact" | "leftovers") {
                // trace: D299
                return Err(self.error(Diagnostic::refused(
                    at(name_offset, base.len()),
                    "`simd_chunks` says what happens to the leftovers: `.pad`, `.exact` or `.leftovers`",
                    "walking in `simd` chunks says how the leftovers are handled: `.pad` fills the last chunk, `.exact` requires a multiple, and `.leftovers` hands them to a `leftovers` section, since dropping them quietly is a classic SIMD bug (D299, D300)",
                )));
            }
            if narrowing == "leftovers" && each.leftovers.is_none() {
                // trace: D300
                return Err(self.error(Diagnostic::refused(
                    each.source.span,
                    "`.leftovers` hands the leftover items to a `leftovers` section, which this loop doesn't have",
                    "the `leftovers` section is required, so the leftovers can't be forgotten (D300)",
                )));
            }
            if narrowing != "leftovers"
                && let Some((name, _)) = &each.leftovers
            {
                return Err(self.error(Diagnostic::refused(
                    name.span,
                    "a `leftovers` section belongs to a `for` over `simd_chunks.leftovers`",
                    "`.leftovers` hands the leftover items to a `leftovers` section, and no other loop has one (D300)",
                )));
            }
            let lanes_argument = arguments
                .iter()
                .find(|argument| argument.label.as_ref().is_some_and(|label| label.text == "lanes"))
                .ok_or_else(|| self.not_yet(each.source.span, "`simd_chunks` without `lanes:`".to_string()))?;
            let lanes = u64::try_from(self.constant_integer(&lanes_argument.value)?).unwrap_or(0).max(1);
            let Some(root) = super::expressions::root_name(receiver).cloned() else {
                return Err(self.not_yet(receiver.span, "chunks of a fresh list".to_string()));
            };
            let Some(reached) = self.reach(body, receiver)? else { return Err(Reported) };
            let (item, known_len) = match &reached.value_type {
                Type::Array { item, len, .. } => ((**item).clone(), Some(*len)),
                Type::List { item, .. } => ((**item).clone(), None),
                Type::Unknown => return Err(Reported),
                other => return Err(self.not_yet(receiver.span, format!("chunks of `{other}`"))),
            };
            let pad = if narrowing == "pad" {
                let argument = arguments
                    .iter()
                    .find(|argument| argument.label.as_ref().is_some_and(|label| label.text == "pad"))
                    .ok_or_else(|| self.not_yet(each.source.span, "`.pad` without `pad:`".to_string()))?;
                Some(self.value_into(body, &argument.value, &item)?)
            } else {
                None
            };
            if changes && !reached.root.mutable {
                return Err(self.error(Diagnostic::refused(
                    receiver.span,
                    format!("`{}` can't change, so it can't be lent with `Lend.mut`", root.text),
                    "`let` declares a variable that can't change, and `let mut` one that can (D3)",
                )));
            }
            if narrowing == "exact"
                && let Some(len) = known_len
                && len % lanes != 0
            {
                return Err(self.error(Diagnostic::refused(
                    each.source.span,
                    format!("this array has {len} items, which isn't a multiple of {lanes} lanes"),
                    "`.exact` requires a multiple, checked when compiling for an array, whose `len` is known (D299)",
                )));
            }
            let word = MachineType::I64;
            let number = |value: u64| checked::Expression {
                kind: Kind::Integer(i128::from(value)),
                machine_type: MachineType::I64,
            };
            let read = |local: checked::LocalId| checked::Expression {
                kind: Kind::Local(local),
                machine_type: MachineType::I64,
            };
            let sum = |left: checked::Expression, right: checked::Expression| checked::Expression {
                kind: Kind::Arithmetic {
                    operator: checked::Arithmetic::WrappingAdd,
                    left: Box::new(left),
                    right: Box::new(right),
                    failure: None,
                    more: Vec::new(),
                },
                machine_type: MachineType::I64,
            };
            let place_at = |position: checked::Expression| {
                let mut steps = reached.steps.clone();
                steps.push(checked::Step::Item { position: Box::new(position), from_one: false, check: None });
                checked::Place { local: reached.root.local, steps }
            };
            let item_at = |position: checked::Expression| checked::Expression {
                kind: Kind::Load(place_at(position)),
                machine_type: item.machine(),
            };
            // The list or array is locked for the whole loop (D303).
            let site = body.site(root.span);
            let flow_place =
                crate::flow::Place { local: reached.root.local, path: Vec::new(), shown: root.text.clone() };
            let spot = crate::flow::Spot { variable: root.span, place: receiver.span, site, marker: site };
            body.events.push(if changes {
                crate::flow::Event::LendMut { place: flow_place, spot }
            } else {
                crate::flow::Event::Read { place: flow_place, spot }
            });
            let count = body.new_local(word.clone());
            let at_item = body.new_local(word.clone());
            let length = match known_len {
                Some(len) => number(len),
                None => checked::Expression {
                    kind: Kind::Convert(Box::new(checked::Expression {
                        kind: Kind::ListLen(Box::new(checked::Expression {
                            kind: Kind::Load(checked::Place {
                                local: reached.root.local,
                                steps: reached.steps.clone(),
                            }),
                            machine_type: reached.value_type.machine(),
                        })),
                        machine_type: MachineType::Integer { bits: 64, signed: false },
                    })),
                    machine_type: word.clone(),
                },
            };
            block.push(checked::Statement::Let { local: count, value: length });
            block.push(checked::Statement::Let { local: at_item, value: number(0) });
            if narrowing == "exact" && known_len.is_none() {
                let check = super::expressions::failure(
                    self,
                    Diagnostic::new(
                        crate::diagnostic::Kind::CheckFailed,
                        span,
                        format!("`{}` isn't a multiple of {lanes} items long", self.slice(receiver.span)),
                        "`.exact` requires a multiple, checked while running for a list (D299)",
                    ),
                    &[
                        super::expressions::When::Text("it holds ".into()),
                        super::expressions::When::Operand(0),
                        super::expressions::When::Text(" items".into()),
                    ],
                );
                let remainder = checked::Expression {
                    kind: Kind::Arithmetic {
                        operator: checked::Arithmetic::RemainderTruncate,
                        left: Box::new(read(count)),
                        right: Box::new(number(lanes)),
                        failure: None,
                        more: Vec::new(),
                    },
                    machine_type: word.clone(),
                };
                block.push(checked::Statement::Check {
                    condition: compare(checked::Comparison::Equal, remainder, number(0)),
                    failure: check,
                    operands: vec![read(count)],
                });
            }
            // The chunks: each round takes `lanes` items, or with `.pad` the last part filled with the pad value.
            let simd = Type::Simd { item: Box::new(item.clone()), lanes };
            let chunk = body.new_local(simd.machine());
            let mut chunk_lanes = Vec::new();
            for index in 0..lanes {
                let position = sum(read(at_item), number(index));
                chunk_lanes.push(match &pad {
                    Some(pad) => {
                        let inside = compare(checked::Comparison::Below, position.clone(), read(count));
                        let clamped = checked::Expression {
                            kind: Kind::Select {
                                condition: Box::new(inside.clone()),
                                then: Box::new(position),
                                otherwise: Box::new(number(0)),
                            },
                            machine_type: word.clone(),
                        };
                        checked::Expression {
                            kind: Kind::Select {
                                condition: Box::new(inside),
                                then: Box::new(item_at(clamped)),
                                otherwise: Box::new(pad.clone()),
                            },
                            machine_type: item.machine(),
                        }
                    }
                    None => item_at(position),
                });
            }
            let keeps_going = if pad.is_some() {
                compare(checked::Comparison::Below, read(at_item), read(count))
            } else {
                compare(checked::Comparison::AtMost, sum(read(at_item), number(lanes)), read(count))
            };
            body.scopes.push(Vec::new());
            let before = body.states.len();
            body.declare(
                Variable {
                    name: each.name.text.clone(),
                    value_type: simd.clone(),
                    mutable: changes,
                    origin: Origin::LoopItem,
                    local: chunk,
                    declared: each.name.span,
                    given: None,
                    known_case: None,
                },
                State::Assigned,
            );
            if !changes {
                body.chunk_copies.push(each.name.text.clone());
            }
            let locked = if changes { &mut body.changing } else { &mut body.reading };
            locked.push(root.text.clone());
            let (id, loop_body, _) = self.loop_body(body, each.label.as_ref(), &each.body);
            if changes {
                body.changing.pop();
            } else {
                body.reading.pop();
                body.chunk_copies.pop();
            }
            body.scopes.pop();
            body.forget_after(before);
            let mut statements = vec![
                checked::Statement::If {
                    condition: not(keeps_going.clone()),
                    then: vec![checked::Statement::Break(id)],
                    otherwise: Vec::new(),
                },
                checked::Statement::Let {
                    local: chunk,
                    value: checked::Expression { kind: Kind::Build(chunk_lanes), machine_type: simd.machine() },
                },
            ];
            statements.extend(loop_body);
            if changes {
                for index in 0..lanes {
                    let position = sum(read(at_item), number(index));
                    let lane = checked::Expression {
                        kind: Kind::Load(checked::Place {
                            local: chunk,
                            steps: vec![checked::Step::Item {
                                position: Box::new(number(index)),
                                from_one: false,
                                check: None,
                            }],
                        }),
                        machine_type: item.machine(),
                    };
                    // Padding lanes are dropped when writing back (D303).
                    statements.push(checked::Statement::If {
                        condition: compare(checked::Comparison::Below, position.clone(), read(count)),
                        then: vec![checked::Statement::Store { place: place_at(position), value: lane }],
                        otherwise: Vec::new(),
                    });
                }
            }
            statements.push(checked::Statement::Set { local: at_item, value: sum(read(at_item), number(lanes)) });
            block.push(checked::Statement::Loop { id, body: statements, opener: body.loop_opener });
            // The leftovers, one at a time, after the chunks, unless a `break` stopped the loop inside them (D300).
            if let Some((name, leftover_body)) = &each.leftovers {
                let leftover = body.new_local(item.machine());
                body.scopes.push(Vec::new());
                let before = body.states.len();
                body.declare(
                    Variable {
                        name: name.text.clone(),
                        value_type: item.clone(),
                        mutable: changes,
                        origin: Origin::LoopItem,
                        local: leftover,
                        declared: name.span,
                        given: None,
                        known_case: None,
                    },
                    State::Assigned,
                );
                let locked = if changes { &mut body.changing } else { &mut body.reading };
                locked.push(root.text.clone());
                let (inner_id, inner_body, _) = self.loop_body(body, None, leftover_body);
                if changes {
                    body.changing.pop();
                } else {
                    body.reading.pop();
                }
                body.scopes.pop();
                body.forget_after(before);
                let mut statements = vec![
                    checked::Statement::If {
                        condition: compare(checked::Comparison::AtLeast, read(at_item), read(count)),
                        then: vec![checked::Statement::Break(inner_id)],
                        otherwise: Vec::new(),
                    },
                    checked::Statement::Let { local: leftover, value: item_at(read(at_item)) },
                ];
                statements.extend(inner_body);
                if changes {
                    statements.push(checked::Statement::Store {
                        place: place_at(read(at_item)),
                        value: checked::Expression { kind: Kind::Local(leftover), machine_type: item.machine() },
                    });
                }
                statements.push(checked::Statement::Set { local: at_item, value: sum(read(at_item), number(1)) });
                block.push(checked::Statement::If {
                    condition: not(keeps_going),
                    then: vec![checked::Statement::Loop { id: inner_id, body: statements, opener: body.loop_opener }],
                    otherwise: Vec::new(),
                });
            }
            Ok(())
        })())
    }

    /// `for $i in range[…]:` (D128, D145, D146). Collections come later.
    fn for_statement(&mut self, body: &mut Body, each: &syntax::For, block: &mut checked::Block) -> Check<()> {
        if let Some(label) = &each.label
            && body.loops.iter().any(|outer| outer.label.as_deref() == Some(label.text.as_str()))
        {
            return Err(self.error(Diagnostic::not_designed(
                label.span,
                format!("naming a loop `{}` inside another with that name", label.text),
                NOT_DECIDED,
            )));
        }
        let is_range = matches!(&each.source.kind, ExpressionKind::Bracket { base, .. }
            if matches!(&base.kind, ExpressionKind::Path(path) if path.is_single("range")));
        if !is_range {
            return self.for_collection(body, each, block);
        }
        let ExpressionKind::Bracket { base, labels, .. } = &each.source.kind else {
            let typed = self.expression(body, &each.source, None)?;
            if matches!(typed.value_type, Type::Integer { .. } | Type::Float { .. } | Type::Bool | Type::Str(_)) {
                return Err(self.error(Diagnostic::refused(
                    each.source.span,
                    format!("`for` goes through a collection or range, and this is `{}`", typed.value_type),
                    "`for` goes through a collection or range, reading it in place (D2, D128)",
                )));
            }
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
                known_case: None,
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
        block.push(checked::Statement::Loop { id, body: statements, opener: body.loop_opener });
        Ok(())
    }

    /// `for $x in $xs:` reads each item in place, `for $x in Lend.mut $xs:` lends each to change, and
    /// `for $x in Give $xs:` hands each out, leaving the list gone (D128, D180, D182).
    #[allow(clippy::too_many_lines)]
    fn for_collection(&mut self, body: &mut Body, each: &syntax::For, block: &mut checked::Block) -> Check<()> {
        use syntax::Ownership;
        let held = super::expressions::is_held(&each.source);
        // trace: D393, D396
        if each.ownership.is_none() && held {
            let shown = self.slice(each.source.span).to_string();
            return Err(self.error(Diagnostic::refused(
                each.source.span,
                format!("this `for` doesn't say how it holds each item of `{shown}`: `Lend`, `Lend.mut` or `Give`"),
                "where there's a choice, the choice is written, and a loop reads, changes or takes each item (D393)",
            )));
        }
        // `Lend` reads each item in place, which is what a fresh collection's bare loop does too.
        let word = each.ownership.map(|(word, _)| word).filter(|word| *word != Ownership::Lend);
        if let Some((Ownership::Lend, _)) = each.ownership
            && !held
        {
            return Err(self.error(Diagnostic::refused(
                each.source.span,
                "a fresh value takes no word",
                "a fresh value that has no holder yet takes no word (D2)",
            )));
        }
        if let Some((Ownership::Copy, span)) = each.ownership {
            return Err(self.error(Diagnostic::not_designed(
                span.to(each.source.span),
                "`Copy` on a loop's collection",
                NOT_DECIDED,
            )));
        }
        if word == Some(Ownership::Give) && !held {
            return Err(self.error(Diagnostic::refused(
                each.source.span,
                "a fresh value takes no word",
                "a fresh value that has no holder yet takes no word (D2)",
            )));
        }
        if word == Some(Ownership::LendMut) && matches!(each.source.kind, ExpressionKind::Field { .. }) {
            let shown = self.slice(each.source.span).to_string();
            return Err(self.error(Diagnostic::not_designed(
                each.source.span,
                format!("whether `for` over `{shown}` uses all of what holds it"),
                "not decided yet (standard 12.1)",
            )));
        }
        let reached = self.reach(body, &each.source)?;
        let (local, steps, collection, root) = match reached {
            Some(reached) => {
                let root = Some((reached.root.clone(), reached.root_span));
                (reached.root.local, reached.steps, reached.value_type, root)
            }
            None => {
                // A fresh list or array goes in a temporary for the loop.
                if let ExpressionKind::Call { callee, .. } = &each.source.kind
                    && let ExpressionKind::Path(path) = &callee.kind
                    && self.resolve_path(path) == "std::coll::list.from"
                {
                    // trace: D396
                    return Err(self.error(Diagnostic::refused(
                        each.source.span,
                        "this fresh list grows on the heap, so it has to end with `done`, which a loop over it can't do",
                        "a loop over a fresh collection takes no word, and one that owns memory is stored first, read with `Lend` and ended with `defer` (D396, D325)",
                    )));
                }
                let typed = self.expression(body, &each.source, None)?;
                if self.must_give(&typed.value_type) {
                    // trace: D396
                    return Err(self.error(Diagnostic::refused(
                        each.source.span,
                        "this fresh list grows on the heap, so it has to end with `done`, which a loop over it can't do",
                        "a loop over a fresh collection takes no word, and one that owns memory is stored first, read with `Lend` and ended with `defer` (D396, D325)",
                    )));
                }
                let local = body.new_local(typed.value.machine_type.clone());
                block.push(checked::Statement::Let { local, value: typed.value });
                (local, Vec::new(), typed.value_type, None)
            }
        };
        let item_type = match &collection {
            Type::Array { item, .. } | Type::List { item, .. } => (**item).clone(),
            Type::Unknown => return Err(Reported),
            Type::New { inner, .. } if matches!(**inner, Type::Array { .. } | Type::List { .. }) => {
                return Err(self.error(Diagnostic::not_designed(
                    each.source.span,
                    "going through a `type.new`, rather than crossing with `as` first",
                    "not decided yet (standard 12.1)",
                )));
            }
            other => {
                if matches!(other, Type::Integer { .. } | Type::Float { .. } | Type::Bool | Type::Str(_)) {
                    return Err(self.error(Diagnostic::refused(
                        each.source.span,
                        format!("`for` goes through a collection or range, and this is `{other}`"),
                        "`for` goes through a collection or range, reading it in place (D2, D128)",
                    )));
                }
                return Err(self.not_yet(each.source.span, "`for` over this".to_string()));
            }
        };
        if let Some((root, root_span)) = &root {
            if word == Some(Ownership::Give) && body.lent.contains(&root.local) {
                return Err(self.error(Diagnostic::refused(
                    *root_span,
                    format!("`{}` is only lent here, so it can't be given away", root.name),
                    "`Lend` is temporary access, and the caller keeps it (D2)",
                )));
            }
            if word == Some(Ownership::LendMut) && !root.mutable {
                return Err(self.error(Diagnostic::refused(
                    *root_span,
                    format!("`{}` can't change, so it can't be lent with `Lend.mut`", root.name),
                    "`let` declares a variable that can't change, and `let mut` one that can (D3)",
                )));
            }
            if !root.value_type.is_copied() {
                let site = body.site(*root_span);
                let marker = each.ownership.map_or(each.source.span, |(_, span)| span.to(each.source.span));
                let place = crate::flow::Place { local: root.local, path: Vec::new(), shown: root.name.clone() };
                let spot = crate::flow::Spot { variable: *root_span, place: each.source.span, site, marker };
                body.events.push(match word {
                    Some(Ownership::Give) => crate::flow::Event::Give { place, spot },
                    Some(Ownership::LendMut) => crate::flow::Event::LendMut { place, spot },
                    _ => crate::flow::Event::Read { place, spot },
                });
            }
        }
        let word_machine = MachineType::Integer { bits: 64, signed: false };
        let index = body.new_local(word_machine.clone());
        let read_index = checked::Expression { kind: Kind::Local(index), machine_type: word_machine.clone() };
        let place_with = |steps: &[checked::Step], item: checked::Step| {
            let mut all = steps.to_vec();
            all.push(item);
            checked::Place { local, steps: all }
        };
        let count = match &collection {
            Type::Array { len, .. } => {
                checked::Expression { kind: Kind::Integer(i128::from(*len)), machine_type: word_machine.clone() }
            }
            _ => checked::Expression {
                kind: Kind::ListLen(Box::new(checked::Expression {
                    kind: Kind::Load(checked::Place { local, steps: steps.clone() }),
                    machine_type: collection.machine(),
                })),
                machine_type: word_machine.clone(),
            },
        };
        block.push(checked::Statement::Let {
            local: index,
            value: checked::Expression { kind: Kind::Integer(0), machine_type: word_machine.clone() },
        });
        let item_step = checked::Step::Item { position: Box::new(read_index.clone()), from_one: false, check: None };
        let item_place = place_with(&steps, item_step);
        let item = body.new_local(item_type.machine());
        let take_item = if word == Some(Ownership::LendMut) {
            checked::Statement::AliasValue {
                local: item,
                value: checked::Expression { kind: Kind::PlaceAddress(item_place), machine_type: MachineType::Address },
            }
        } else {
            checked::Statement::Let {
                local: item,
                value: checked::Expression { kind: Kind::Load(item_place), machine_type: item_type.machine() },
            }
        };
        body.scopes.push(Vec::new());
        let before = body.states.len();
        if word != Some(Ownership::Give) {
            body.lent.push(item);
        }
        body.declare(
            Variable {
                name: each.name.text.clone(),
                value_type: item_type.clone(),
                mutable: word == Some(Ownership::LendMut),
                origin: Origin::LoopItem,
                local: item,
                declared: each.name.span,
                given: None,
                known_case: None,
            },
            State::Assigned,
        );
        let mut binding = Vec::new();
        if word == Some(Ownership::Give) && !item_type.is_copied() {
            binding.push(crate::flow::Event::Bind {
                local: item,
                name: each.name.text.clone(),
                declared: each.name.span,
                must_give: self.must_give(&item_type),
                origin: crate::flow::Origin::Local,
                why: self.duty_why(&item_type),
            });
        }
        let reading = root.as_ref().map(|(root, _)| root.name.clone());
        let changing = word == Some(Ownership::LendMut);
        let part = !steps.is_empty();
        if changing {
            body.changing.extend(reading.clone());
        } else if part {
            // The part as written, `$b.items`, so changing that part and changing another can be told apart (D125).
            let written = self.slice(each.source.span).to_string();
            body.reading_parts.extend(reading.as_ref().map(|_| written));
        } else {
            body.reading.extend(reading.clone());
        }
        let giving = word == Some(Ownership::Give) && self.must_give(&item_type);
        if giving && let Some((root, _)) = &root {
            let given_span = each.ownership.map_or(each.source.span, |(_, span)| span.to(each.source.span));
            body.give_loops.push((body.loops.len(), root.name.clone(), given_span));
        }
        let start = body.states[..before].to_vec();
        let (id, mut loop_body, breaks) = self.loop_body(body, each.label.as_ref(), &each.body);
        if giving && root.is_some() {
            body.give_loops.pop();
        }
        if reading.is_some() {
            if changing {
                body.changing.pop();
            } else if part {
                body.reading_parts.pop();
            } else {
                body.reading.pop();
            }
        }
        // Each item a `Give` loop hands out has to be given away or finished in its round (D180).
        if word == Some(Ownership::Give) && self.must_give(&item_type) {
            loop_body.push(checked::Statement::Flow(vec![crate::flow::Event::Expire { local: item }]));
        }
        body.scopes.pop();
        body.forget_after(before);
        let mut joined = Body::join_states(&start, &body.states[..before]);
        for state in breaks {
            joined = Body::join_states(&joined, &state[..before]);
        }
        body.states = joined;
        let mut statements = vec![
            checked::Statement::If {
                condition: not(compare(checked::Comparison::Below, read_index.clone(), count)),
                then: vec![checked::Statement::Break(id)],
                otherwise: Vec::new(),
            },
            take_item,
            checked::Statement::Flow(binding),
        ];
        statements.extend(loop_body);
        statements.push(checked::Statement::Set {
            local: index,
            value: checked::Expression {
                kind: Kind::Arithmetic {
                    operator: checked::Arithmetic::WrappingAdd,
                    left: Box::new(read_index),
                    right: Box::new(checked::Expression { kind: Kind::Integer(1), machine_type: word_machine.clone() }),
                    failure: None,
                    more: Vec::new(),
                },
                machine_type: word_machine,
            },
        });
        block.push(checked::Statement::Loop { id, body: statements, opener: body.loop_opener });
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
                // A step can be any integer, and doesn't have to match the range's type (D146).
                let typed = self.expression(body, &step.value, None)?;
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
            // A range ending before where it starts is empty, which still ends; only one going the wrong way can't.
            let reachable = if by > 0 { start <= stop } else { start >= stop };
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
            Type::Integer { unit: Some(unit), .. } if *unit != super::types::Unit::named("items") => Err(self.error(
                Diagnostic::not_designed(value.span, "a range of numbers with a unit other than `items`", NOT_DECIDED),
            )),
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
        if let Ok(typed) = &from_result {
            self.range_number(typed, from)?;
        }
        let end_result = self.expression(body, end, from_result.as_ref().ok().map(|typed| &typed.value_type));
        let (mut from_typed, mut end_typed) = (from_result?, end_result?);
        self.range_number(&end_typed, end)?;
        // A literal end takes the other's type, and its unit, as `range[from: 1, to: $xs.len]` does (D142).
        if from_typed.literal.is_some() && end_typed.literal.is_none() {
            let unit = if from_typed.value_type.unit().is_none() { end_typed.value_type.unit().cloned() } else { None };
            from_typed = self.expression(body, from, Some(&end_typed.value_type.plain()))?;
            if unit.is_some() {
                from_typed.value_type = from_typed.value_type.with_unit(unit);
            }
        } else if end_typed.literal.is_some() && from_typed.literal.is_none() {
            let unit = if end_typed.value_type.unit().is_none() { from_typed.value_type.unit().cloned() } else { None };
            end_typed = self.expression(body, end, Some(&from_typed.value_type.plain()))?;
            if unit.is_some() {
                end_typed.value_type = end_typed.value_type.with_unit(unit);
            }
        }
        // A plain variable at one end and a number in `items` at the other isn't decided.
        let same_width = from_typed.value_type.with_unit(None).plain() == end_typed.value_type.with_unit(None).plain();
        if same_width && from_typed.value_type.unit().is_some() != end_typed.value_type.unit().is_some() {
            let (plain, plain_span) =
                if from_typed.value_type.unit().is_none() { (&from_typed, from.span) } else { (&end_typed, end.span) };
            if plain.literal.is_none() {
                return Err(self.error(Diagnostic::not_designed(
                    plain_span,
                    "a range mixing a plain variable with a number in `items`",
                    NOT_DECIDED,
                )));
            }
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

/// A block statement's opener: from its start to the `:` ending its first line.
fn opener(text: &str, span: Span) -> Span {
    let start = span.start as usize;
    let line_end =
        text.get(start..).and_then(|rest| rest.find('\n')).map_or(span.end as usize, |offset| start + offset);
    let line = text.get(start..line_end).unwrap_or("");
    let end = line.rfind(':').map_or(line_end, |colon| start + colon + 1);
    Span::new(span.file, start, end)
}

/// Whether a place goes through an item, as in `$b.boxes[pos: 1].n`.
fn has_item(expression: &syntax::Expression) -> bool {
    match &expression.kind {
        ExpressionKind::Bracket { base, .. } => !matches!(base.kind, ExpressionKind::Path(_)),
        ExpressionKind::Field { base, .. } | ExpressionKind::Group(base) => has_item(base),
        _ => false,
    }
}

/// How many bits a register holds, by its chip's names: `x0` and `w0` on aarch64, `rax` and `eax` on x86-64.
fn register_bits(chip: &str, register: &str) -> Option<u32> {
    match chip {
        "aarch64" => match register.chars().next() {
            Some('x') if register[1..].parse::<u32>().is_ok_and(|number| number <= 30) => Some(64),
            Some('w') if register[1..].parse::<u32>().is_ok_and(|number| number <= 30) => Some(32),
            _ => None,
        },
        "x86_64" => match register {
            "rax" | "rbx" | "rcx" | "rdx" | "rsi" | "rdi" | "rbp" | "r8" | "r9" | "r10" | "r11" | "r12" | "r13"
            | "r14" | "r15" => Some(64),
            "eax" | "ebx" | "ecx" | "edx" | "esi" | "edi" | "ebp" => Some(32),
            _ => None,
        },
        _ => None,
    }
}

/// The variable a part is of: `$b` for `$b.items` or `$xs[pos: 1]`.
pub fn part_root(part: &str) -> &str {
    part.split(['.', '[']).next().unwrap_or(part)
}
