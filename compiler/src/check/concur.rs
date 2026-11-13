//! `@concur` (D343 to D360, D375 to D377, D380, D383, D384, D392): structured tasks, runners, locks and atomics.
//!
//! Tasks keep Cyborg's answers the same on every run (D347), so this compiler runs each one where the standard says it
//! may: a task on `os::threads` as it's spawned, and one on `std::run::one_thread` or `os::events` when the code that
//! spawned it first waits, at `@concur::task.done`, with the tasks before it in the order they were spawned (D376).
//! Nothing runs at the same time yet, which no answer can tell apart, since tasks share nothing that races (D347).

use super::expressions::{Typed, When, failure, root_name};
use super::types::{RunnerKind, Type};
use super::{Body, Check, Checker, Reported};
use crate::checked::{self, ExpressionKind as Kind, LocalId, MachineType};
use crate::diagnostic::{Diagnostic, Kind as DiagnosticKind};
use crate::source::Span;
use crate::syntax::{self, ExpressionKind, Ownership};

const STACK_WHY: &str = "each task on a runner that switches tasks has its own stack, from the allocator the spawn names, with its size written, as in `alloc: Lend $heap, stack: 4 kib` (D351, D352, D384, D392)";
const ATOMIC_MARK_WHY: &str =
    "an atomic is shared with plain `Lend` and changed by indivisible instructions, so its operations take `.%` (D354)";

/// A task this function spawned: the call it runs, where its result goes, and whether it has run yet.
#[derive(Clone)]
pub struct Task {
    pub signature: super::Signature,
    pub call: checked::Expression,
    pub result: LocalId,
    pub ran: LocalId,
    /// Spawned onto a runner that waits to run it, rather than at once.
    pub waits: bool,
    pub order: usize,
}

/// A task just spawned, waiting for the `let` that holds it, which borrows what the call borrows.
pub struct Spawned {
    pub task: Task,
    pub borrows: Vec<(LocalId, Vec<usize>)>,
    pub mutable: bool,
}

impl Checker<'_> {
    /// `@concur::task<i64>`, `@concur::lock<i64>`, `@concur::atomic<u64>`, and the runners (D344, D345, D348, D354,
    /// D359, D377).
    pub fn concur_type(
        &mut self,
        written: &syntax::Type,
        text: &str,
        fill: &[syntax::TypeArgument],
    ) -> Option<Check<Type>> {
        let runner = match text {
            "std::run::one_thread" => Some(RunnerKind::OneThread),
            "os::threads" => Some(RunnerKind::Threads),
            "os::events" => Some(RunnerKind::Events),
            _ => None,
        };
        if let Some(kind) = runner {
            let span = match &written.kind {
                syntax::TypeKind::Named { path, .. } => path.span,
                _ => written.span,
            };
            return Some(self.require_import(text, span).map(|()| Type::Runner(kind)));
        }
        if !matches!(text, "@concur::task" | "@concur::lock" | "@concur::atomic") {
            return None;
        }
        Some((|| {
            let span = match &written.kind {
                syntax::TypeKind::Named { path, .. } => path.span,
                _ => written.span,
            };
            self.require_import(text, span)?;
            let [syntax::TypeArgument { label: None, value: syntax::TypeArgumentValue::Type(inner), .. }] = fill else {
                return Err(self.not_yet(written.span, format!("`{text}` filled in this way")));
            };
            let inner_span = inner.span;
            let inner = self.resolve_type(inner)?;
            match text {
                "@concur::task" => Ok(Type::Task(Box::new(inner))),
                "@concur::lock" => Ok(Type::Lock(Box::new(inner))),
                _ => {
                    if !matches!(inner, Type::Integer { .. } | Type::Bool) {
                        // trace: D354
                        return Err(self.error(Diagnostic::refused(
                            inner_span,
                            format!("an atomic holds an integer or a `bool`, and this is `{inner}`"),
                            "an atomic is changed by the CPU's own indivisible instructions, which work on integers and `bool` (D354)",
                        )));
                    }
                    Ok(Type::Atomic(Box::new(inner)))
                }
            }
        })())
    }

    /// `std::run::one_thread()`, `os::threads(count: 8)`, `os::events(count: 4, alloc: Lend $heap)`,
    /// `@concur::lock.new(…)`, `@concur::atomic.new(…)`, `@concur::lock.done(Give $l)` and
    /// `@concur::task.done(Give $t)`, or `None` for any other call.
    pub fn concur_call(
        &mut self,
        body: &mut Body,
        expression: &syntax::Expression,
        name: &str,
        arguments: &[syntax::Argument],
        expected: Option<&Type>,
    ) -> Option<Check<Typed>> {
        let made = match name {
            "std::run::one_thread" => Some(RunnerKind::OneThread),
            "os::threads" => Some(RunnerKind::Threads),
            "os::events" => Some(RunnerKind::Events),
            _ => None,
        };
        if let Some(kind) = made {
            return Some(self.runner(body, expression, kind, arguments));
        }
        match name {
            "@concur::lock.new" | "@concur::atomic.new" => Some((|| {
                let [argument] = arguments else {
                    return Err(self.not_yet(expression.span, format!("`{name}` with these arguments")));
                };
                let (inner, wrap): (Type, fn(Box<Type>) -> Type) = match (name, expected) {
                    ("@concur::lock.new", Some(Type::Lock(inner))) => ((**inner).clone(), Type::Lock),
                    ("@concur::atomic.new", Some(Type::Atomic(inner))) => ((**inner).clone(), Type::Atomic),
                    _ => return Err(self.not_yet(expression.span, format!("`{name}` where nothing says its type"))),
                };
                let value = match argument.ownership {
                    Some((Ownership::Give, _)) => {
                        if let Some(root) = root_name(&argument.value).cloned() {
                            self.give_away(body, &root, &argument.value)?;
                        }
                        self.expression(body, &argument.value, Some(&inner))?.value
                    }
                    _ => self.value_into(body, &argument.value, &inner)?,
                };
                Ok(Typed::new(wrap(Box::new(inner)), value.kind))
            })()),
            "@concur::lock.done" => Some((|| {
                let [argument] = arguments else {
                    return Err(self.not_yet(expression.span, "`@concur::lock.done` with these arguments".to_string()));
                };
                let typed = self.expression(body, &argument.value, None)?;
                let Type::Lock(inner) = typed.value_type.clone() else {
                    return Err(
                        self.not_yet(argument.span, "`@concur::lock.done` on something that isn't a lock".to_string())
                    );
                };
                if let Some(root) = root_name(&argument.value).cloned() {
                    self.give_away(body, &root, &argument.value)?;
                }
                Ok(Typed::new(*inner, typed.value.kind))
            })()),
            "@concur::task.done" => Some((|| {
                let (signature, value) = self.task_done(body, expression, arguments)?;
                if signature.failures.is_empty() {
                    return Ok(Typed::new(signature.returns.clone(), value.kind));
                }
                Ok(self.crash_on_failure(expression.span, &signature, value))
            })()),
            _ => None,
        }
    }

    /// The runners: `std::run::one_thread()`, `os::threads(count: …)` and `os::events(count: …, alloc: …)` (D345,
    /// D346, D350, D377, D392).
    fn runner(
        &mut self,
        body: &mut Body,
        expression: &syntax::Expression,
        kind: RunnerKind,
        arguments: &[syntax::Argument],
    ) -> Check<Typed> {
        let labelled =
            |label: &str| arguments.iter().find(|each| each.label.as_ref().is_some_and(|name| name.text == label));
        match kind {
            RunnerKind::OneThread => {
                if let Some(argument) = arguments.first() {
                    return Err(self.not_yet(argument.span, "an argument to `std::run::one_thread()`".to_string()));
                }
            }
            RunnerKind::Threads | RunnerKind::Events => {
                let wants_alloc = kind == RunnerKind::Events;
                let (count, alloc) = (labelled("count"), labelled("alloc"));
                if count.is_none() || (wants_alloc && alloc.is_none()) {
                    let (name, what) =
                        if wants_alloc { ("os::events", "`count:` and `alloc:`") } else { ("os::threads", "`count:`") };
                    return Err(self.error(Diagnostic::refused(
                        expression.span,
                        format!("`{name}` takes {what}"),
                        "the thread count is always written, a number or `max` for every core, so a program behaves the same everywhere unless it says otherwise, and an event runner also takes the allocator for its queue (D345, D346, D392)",
                    )));
                }
                if let Some(count) = count
                    && !matches!(&count.value.kind, ExpressionKind::Path(path) if path.is_single("max"))
                {
                    let number = self.constant_integer(&count.value)?;
                    if number < 1 {
                        return Err(self.error(Diagnostic::refused(
                            count.value.span,
                            "a runner has at least one thread",
                            "`count:` takes a number from 1, or `max` for every core (D345, D346)",
                        )));
                    }
                }
                if let Some(alloc) = alloc {
                    self.allocator_argument(body, alloc)?;
                }
            }
        }
        Ok(Typed::new(Type::Runner(kind), Kind::Integer(0)))
    }

    /// `@concur::spawn(on: Lend $runner, …) call(…)`: a call started as a task (D343, D345, D351, D352, D356, D384).
    #[allow(clippy::too_many_lines)]
    pub fn spawn(
        &mut self,
        body: &mut Body,
        expression: &syntax::Expression,
        options: &[syntax::Argument],
        options_span: Span,
        call: &syntax::Expression,
    ) -> Check<Typed> {
        self.require_import("@concur::spawn", options_span)?;
        let labelled =
            |label: &str| options.iter().find(|each| each.label.as_ref().is_some_and(|name| name.text == label));
        let Some(on) = labelled("on") else {
            // trace: D345
            return Err(self.error(Diagnostic::refused(
                expression.span,
                "`@concur::spawn` names its runner with `on:`",
                "every spawn names where its task runs, as every `append` names its allocator (D345)",
            )));
        };
        let runner = self.expression(body, &on.value, None)?;
        let Type::Runner(kind) = runner.value_type else {
            if runner.value_type.is_unknown() {
                return Err(Reported);
            }
            return Err(self.error(Diagnostic::refused(
                on.value.span,
                format!("this is `{}`, where a runner is needed", runner.value_type),
                "a task runs on a runner: `std::run::one_thread`, `os::threads` or `os::events` (D345, D359, D377)",
            )));
        };
        let (alloc, stack) = (labelled("alloc"), labelled("stack"));
        match kind {
            RunnerKind::Threads => {
                if let Some(extra) = alloc.or(stack) {
                    // trace: D351
                    return Err(self.error(Diagnostic::refused(
                        extra.value.span,
                        "a spawn onto `os::threads` takes neither `alloc:` nor `stack:`",
                        "the OS gives a pool's threads their stacks, so nothing is allocated here (D351, D384)",
                    )));
                }
            }
            RunnerKind::OneThread | RunnerKind::Events => {
                let (Some(alloc), Some(stack)) = (alloc, stack) else {
                    // trace: D384, D392
                    return Err(self.error(Diagnostic::refused(
                        expression.span,
                        format!("a spawn onto `{}` takes `alloc:` and `stack:`", Type::Runner(kind)),
                        STACK_WHY,
                    )));
                };
                self.allocator_argument(body, alloc)?;
                self.byte_size(&stack.value)?;
            }
        }
        if let ExpressionKind::Pass { value, .. } = &call.kind {
            // trace: D357
            let shown = self.slice(value.span).to_string();
            return Err(self.error(Diagnostic::refused_fix(
                call.span,
                shown,
                "`@concur::task.done(Give $t)?` passes a task's failure up (D343, D357)",
            )));
        }
        let ExpressionKind::Call { callee, arguments, .. } = &call.kind else {
            return Err(self.not_yet(call.span, "spawning something that isn't a call".to_string()));
        };
        // What the call borrows, the task holds until it's done (D343, D347).
        let mut borrows = Vec::new();
        let mut mutable = false;
        for argument in arguments {
            let word = argument.ownership.map(|(word, _)| word);
            if matches!(word, Some(Ownership::Lend | Ownership::LendMut))
                && let Some((local, path)) = self.any_place(body, &argument.value)
            {
                mutable |= word == Some(Ownership::LendMut);
                borrows.push((local, path));
            }
        }
        let (signature, checked_call) = self.raw_call(body, call, callee, arguments)?;
        // The arguments are taken where the spawn is written (D356).
        let Kind::Call { function, arguments: taken, .. } = checked_call.kind.clone() else { return Err(Reported) };
        let mut held = Vec::new();
        for argument in taken {
            held.push(match argument {
                checked::Argument::Value(value) | checked::Argument::Temporary(value) => {
                    let local = body.new_local(value.machine_type.clone());
                    let machine_type = value.machine_type.clone();
                    body.before.push(checked::Statement::Let { local, value });
                    checked::Argument::Value(checked::Expression { kind: Kind::Local(local), machine_type })
                }
                other => other,
            });
        }
        let call_kind = match checked_call.kind {
            Kind::Call { function: _, arguments: _, span } => Kind::Call { function, arguments: held, span },
            other => other,
        };
        let deferred = checked::Expression { kind: call_kind, machine_type: checked_call.machine_type.clone() };
        let result = body.new_local(deferred.machine_type.clone());
        let ran = body.new_local(MachineType::Bool);
        let waits = kind != RunnerKind::Threads;
        if waits {
            body.before.push(checked::Statement::Let {
                local: ran,
                value: checked::Expression { kind: Kind::Bool(false), machine_type: MachineType::Bool },
            });
        } else {
            // A task on the OS's threads starts as it's spawned.
            body.before.push(checked::Statement::Let { local: result, value: deferred.clone() });
            body.before.push(checked::Statement::Let {
                local: ran,
                value: checked::Expression { kind: Kind::Bool(true), machine_type: MachineType::Bool },
            });
        }
        let order = body.task_count;
        body.task_count += 1;
        let task = Task { signature: signature.clone(), call: deferred, result, ran, waits, order };
        self.spawned = Some(Spawned { task, borrows, mutable });
        let _ = expression;
        Ok(Typed::new(Type::Task(Box::new(signature.returns.clone())), Kind::Integer(0)))
    }

    /// `@concur::task.done(Give $t)`: runs the task if it's still waiting, with the ones spawned before it, and gives
    /// its result, its failure still to handle (D357, D376).
    pub fn task_done(
        &mut self,
        body: &mut Body,
        expression: &syntax::Expression,
        arguments: &[syntax::Argument],
    ) -> Check<(super::Signature, checked::Expression)> {
        let [argument] = arguments else {
            return Err(self.not_yet(expression.span, "`@concur::task.done` with these arguments".to_string()));
        };
        let Some(root) = root_name(&argument.value).cloned() else {
            return Err(self.not_yet(argument.span, "`@concur::task.done` on a fresh task".to_string()));
        };
        let Some(variable) = body.lookup(&root.text).cloned() else { return Err(Reported) };
        // A task whose `let` was refused never got a value to finish (D203).
        if let Some(index) = body.find(&root.text)
            && body.states[index] != super::State::Assigned
            && variable.origin == super::Origin::RefusedLet
        {
            if variable.value_type.is_unknown() {
                return Err(Reported);
            }
            return Err(self.error(Diagnostic::refused(
                expression.span,
                format!("`{}` never got a value", root.text),
                "a variable is read only after it has one (D203)",
            )));
        }
        self.require_import("@concur::task.done", expression.span)?;
        let Some(task) = body.tasks.iter().find(|(local, _)| *local == variable.local).map(|(_, task)| task.clone())
        else {
            return Err(self.error(Diagnostic::not_designed(
                argument.span,
                "what a task can fail with once it's handed to another function, since a function's type doesn't carry its failures",
                "not decided yet (standard 12.2)",
            )));
        };
        self.give_away(body, &root, &argument.value)?;
        // The runner switches to waiting tasks in the order they were spawned (D376).
        let mut waiting: Vec<Task> = body
            .tasks
            .iter()
            .map(|(_, each)| each.clone())
            .filter(|each| each.waits && each.order <= task.order)
            .collect();
        waiting.sort_by_key(|each| each.order);
        for each in waiting {
            let read_ran = checked::Expression { kind: Kind::Local(each.ran), machine_type: MachineType::Bool };
            body.before.push(checked::Statement::If {
                condition: checked::Expression { kind: Kind::Not(Box::new(read_ran)), machine_type: MachineType::Bool },
                then: vec![
                    checked::Statement::Let { local: each.result, value: each.call.clone() },
                    checked::Statement::Set {
                        local: each.ran,
                        value: checked::Expression { kind: Kind::Bool(true), machine_type: MachineType::Bool },
                    },
                ],
                otherwise: Vec::new(),
            });
        }
        let value =
            checked::Expression { kind: Kind::Local(task.result), machine_type: task.call.machine_type.clone() };
        Ok((task.signature, value))
    }

    /// `@concur::locked $a, $b as $x, $y: … end @concur::locked`: every lock taken for the block, never inside another
    /// (D348, D349).
    pub fn locked(
        &mut self,
        body: &mut Body,
        span: Span,
        (locks, names, inner): (&[syntax::Expression], &[syntax::Name], &[syntax::Statement]),
        block: &mut checked::Block,
    ) -> Check<bool> {
        if body.locked > 0 {
            // trace: D348
            return Err(self.error(Diagnostic::refused(
                span,
                "this locks inside a locked block",
                "a deadlock needs a task holding one lock while waiting for another, so locking inside a locked block is refused, and one block takes every lock it needs (D348, D349)",
            )));
        }
        let mut seen: Vec<String> = Vec::new();
        for lock in locks {
            let shown = self.slice(lock.span).to_string();
            if seen.contains(&shown) {
                // trace: D349
                return Err(self.error(Diagnostic::refused(
                    lock.span,
                    format!("`{shown}` is locked twice here"),
                    "a task taking one lock twice would wait on itself (D349)",
                )));
            }
            seen.push(shown);
        }
        if locks.len() != names.len() {
            return Err(
                self.not_yet(span, "a `@concur::locked` naming a different count of locks and names".to_string())
            );
        }
        body.scopes.push(Vec::new());
        let start = body.states.len();
        for (lock, name) in locks.iter().zip(names) {
            let Some(reached) = self.reach(body, lock)? else {
                return Err(self.not_yet(lock.span, "locking this".to_string()));
            };
            let Type::Lock(inner) = &reached.value_type else {
                if reached.value_type.is_unknown() {
                    return Err(Reported);
                }
                return Err(self.error(Diagnostic::refused(
                    lock.span,
                    format!("this is `{}`, where a lock is needed", reached.value_type),
                    "`@concur::locked` takes locks, `@concur::lock<…>`, and lends what each holds to change (D348)",
                )));
            };
            let local = body.new_local(inner.machine());
            let path: Vec<usize> = reached
                .steps
                .iter()
                .filter_map(|step| match step {
                    checked::Step::Field(index) => Some(*index),
                    checked::Step::Item { .. } => None,
                })
                .collect();
            block.push(checked::Statement::Alias { local, target: reached.root.local, path });
            body.declare(
                super::Variable {
                    name: name.text.clone(),
                    value_type: (**inner).clone(),
                    mutable: true,
                    origin: super::Origin::Let,
                    local,
                    declared: name.span,
                    given: None,
                    known_case: None,
                },
                super::State::Assigned,
            );
        }
        body.locked += 1;
        let checked = self.block(body, inner);
        body.locked -= 1;
        body.scopes.pop();
        body.forget_after(start);
        block.extend(checked.block);
        Ok(checked.leaves)
    }

    /// An atomic's operations: `%load`, `%add`, `%sub`, `%and.bit`, `%or.bit`, `%xor.bit` and `%swap`, each taking an
    /// ordering narrowing, as in `.%add.relaxed` (D354, D360, D361).
    #[allow(clippy::too_many_lines)]
    pub fn atomic_method(
        &mut self,
        body: &mut Body,
        expression: &syntax::Expression,
        receiver: &syntax::Expression,
        method: &syntax::Path,
        arguments: &[syntax::Argument],
    ) -> Check<Typed> {
        const ORDERINGS: [&str; 5] = ["relaxed", "acquire", "release", "acq_rel", "seq_cst"];
        let written = method.text();
        let Some(reached) = self.reach(body, receiver)? else { return Err(Reported) };
        let Type::Atomic(inner) = reached.value_type.clone() else { return Err(Reported) };
        let inner = *inner;
        if written.starts_with("%%") {
            // trace: D354
            return Err(self.error(Diagnostic::refused_fix(method.span, written[1..].to_string(), ATOMIC_MARK_WHY)));
        }
        let parts: Vec<&str> = written.trim_start_matches('%').split('.').collect();
        let operation = parts.first().copied().unwrap_or("");
        let narrowings: Vec<&str> = parts[1..].iter().copied().filter(|part| !ORDERINGS.contains(part)).collect();
        if matches!(operation, "and" | "or" | "xor") && narrowings.first() != Some(&"bit") {
            // trace: D361
            let start = method.span.start as usize + 1;
            let name_span = Span::new(method.span.file, start, start + operation.len());
            return Err(self.error(Diagnostic::refused_fix(
                name_span,
                format!("{operation}.bit"),
                "bit operators say `.bit`, so they can't be taken for logic (D361)",
            )));
        }
        let place = checked::Place { local: reached.root.local, steps: reached.steps.clone() };
        let path: Vec<usize> = reached
            .steps
            .iter()
            .filter_map(|step| match step {
                checked::Step::Field(index) => Some(*index),
                checked::Step::Item { .. } => None,
            })
            .collect();
        let address = checked::Expression {
            kind: Kind::AddressOf { local: reached.root.local, path },
            machine_type: MachineType::Address,
        };
        let current = checked::Expression { kind: Kind::Load(place), machine_type: inner.machine() };
        if operation == "load" {
            return Ok(Typed::new(inner, current.kind));
        }
        let labelled =
            |label: &str| arguments.iter().find(|each| each.label.as_ref().is_some_and(|name| name.text == label));
        let Some(given) = labelled("val") else {
            return Err(self.not_yet(expression.span, format!("`{written}` without `val:`")));
        };
        let value = self.value_into(body, &given.value, &inner.plain())?;
        let operator = match (operation, narrowings.first().copied()) {
            ("swap", None) => None,
            ("add", None) => Some(checked::Arithmetic::Add),
            ("add", Some("wrap")) => Some(checked::Arithmetic::WrappingAdd),
            ("add", Some("saturate")) => Some(checked::Arithmetic::SaturatingAdd),
            ("sub", None) => Some(checked::Arithmetic::Subtract),
            ("sub", Some("wrap")) => Some(checked::Arithmetic::WrappingSubtract),
            ("sub", Some("saturate")) => Some(checked::Arithmetic::SaturatingSubtract),
            ("and", Some("bit")) => Some(checked::Arithmetic::BitAnd),
            ("or", Some("bit")) => Some(checked::Arithmetic::BitOr),
            ("xor", Some("bit")) => Some(checked::Arithmetic::BitXor),
            _ => return Err(self.not_yet(method.span, format!("the atomic operation `{written}`"))),
        };
        let held = body.new_local(inner.machine());
        let read_held = checked::Expression { kind: Kind::Local(held), machine_type: inner.machine() };
        let new_value = match operator {
            None => value,
            Some(operator) => {
                let checks = matches!(operator, checked::Arithmetic::Add | checked::Arithmetic::Subtract);
                let failure = checks.then(|| {
                    let name = format!(".%{operation}");
                    let type_name = inner.plain().to_string();
                    Box::new(failure(
                        self,
                        Diagnostic::new(
                            DiagnosticKind::CheckFailed,
                            expression.span,
                            format!("this `{name}` overflows `{type_name}`"),
                            "an atomic's `.%add` is checked for overflow like `+`, with `.wrap` and `.saturate` when meant (D42, D354, D360)",
                        ),
                        &[
                            When::Text("`".into()),
                            When::Operand(0),
                            When::Text("` and `".into()),
                            When::Operand(1),
                            When::Text("`".into()),
                        ],
                    ))
                });
                checked::Expression {
                    kind: Kind::Arithmetic {
                        operator,
                        left: Box::new(read_held.clone()),
                        right: Box::new(value),
                        failure,
                        more: Vec::new(),
                    },
                    machine_type: inner.machine(),
                }
            }
        };
        let write = checked::Expression {
            kind: Kind::PointerWrite { pointer: Box::new(address), value: Box::new(new_value), volatile: false },
            machine_type: MachineType::Address,
        };
        let ignored = body.new_local(MachineType::Address);
        // The old value comes back from `swap`, and nothing from the others.
        let (value_type, after) = if operation == "swap" {
            (inner.clone(), read_held)
        } else {
            (Type::Void, checked::Expression { kind: Kind::Local(ignored), machine_type: MachineType::Address })
        };
        let inner_bind = checked::Expression {
            kind: Kind::Bind { local: ignored, value: Box::new(write), body: Box::new(after) },
            machine_type: value_type.machine(),
        };
        let whole = checked::Expression {
            kind: Kind::Bind { local: held, value: Box::new(current), body: Box::new(inner_bind) },
            machine_type: value_type.machine(),
        };
        Ok(Typed::new(value_type, whole.kind))
    }

    /// `$owner.%compare_swap(expect: 0, val: $me)` as a statement: sets the value only if it's still `expect`, and
    /// otherwise fails with `not_expected[val: …]`, carrying what's there now (D360). `None` for anything else.
    pub fn compare_swap_statement(
        &mut self,
        body: &mut Body,
        expression: &syntax::Expression,
        block: &mut checked::Block,
    ) -> Option<Check<()>> {
        let (call, arms, ignored) = match &expression.kind {
            ExpressionKind::Otherwise { value, handling: syntax::Handling::Arms { arms, .. }, .. } => {
                (value.as_ref(), Some(arms), false)
            }
            ExpressionKind::Instead { value, .. } => (value.as_ref(), None, true),
            _ => (expression, None, false),
        };
        let ExpressionKind::MethodCall { receiver, method, arguments, .. } = &call.kind else { return None };
        if !method.text().starts_with("%compare_swap") {
            return None;
        }
        Some((|| {
            let Some(reached) = self.reach(body, receiver)? else { return Err(Reported) };
            let Type::Atomic(inner) = reached.value_type.clone() else { return Err(Reported) };
            let inner = *inner;
            let labelled =
                |label: &str| arguments.iter().find(|each| each.label.as_ref().is_some_and(|name| name.text == label));
            let (Some(expect), Some(given)) = (labelled("expect"), labelled("val")) else {
                return Err(self.not_yet(call.span, "`%compare_swap` without `expect:` and `val:`".to_string()));
            };
            let expect = self.value_into(body, &expect.value, &inner.plain())?;
            let given = self.value_into(body, &given.value, &inner.plain())?;
            let path: Vec<usize> = reached
                .steps
                .iter()
                .filter_map(|step| match step {
                    checked::Step::Field(index) => Some(*index),
                    checked::Step::Item { .. } => None,
                })
                .collect();
            let address = checked::Expression {
                kind: Kind::AddressOf { local: reached.root.local, path },
                machine_type: MachineType::Address,
            };
            let place = checked::Place { local: reached.root.local, steps: reached.steps.clone() };
            let now = body.new_local(inner.machine());
            let read_now = checked::Expression { kind: Kind::Local(now), machine_type: inner.machine() };
            block.push(checked::Statement::Let {
                local: now,
                value: checked::Expression { kind: Kind::Load(place), machine_type: inner.machine() },
            });
            let same = checked::Expression {
                kind: Kind::Compare {
                    operator: checked::Comparison::Equal,
                    left: Box::new(read_now.clone()),
                    right: Box::new(expect),
                },
                machine_type: MachineType::Bool,
            };
            let write = checked::Statement::Evaluate(checked::Expression {
                kind: Kind::PointerWrite { pointer: Box::new(address), value: Box::new(given), volatile: false },
                machine_type: MachineType::Address,
            });
            let mut otherwise = Vec::new();
            match arms {
                Some(arms) => {
                    let before = body.states.clone();
                    for arm in arms.iter() {
                        let name = arm.pattern.text();
                        if name != "not_expected" {
                            return Err(self.error(Diagnostic::refused(
                                arm.pattern.span,
                                format!("`%compare_swap` fails only with `not_expected`, not `{name}`"),
                                "a lost race fails with `not_expected[val: …]`, carrying what's there now (D360)",
                            )));
                        }
                        body.scopes.push(Vec::new());
                        let start = body.states.len();
                        if let Some(binding) = &arm.binding {
                            let id = self.not_expected_struct(&inner);
                            let failed = body.new_local(MachineType::Struct(id));
                            otherwise.push(checked::Statement::Let {
                                local: failed,
                                value: checked::Expression {
                                    kind: Kind::Build(vec![read_now.clone()]),
                                    machine_type: MachineType::Struct(id),
                                },
                            });
                            body.declare(
                                super::Variable {
                                    name: binding.text.clone(),
                                    value_type: Type::Struct { id, name: "not_expected".to_string() },
                                    mutable: false,
                                    origin: super::Origin::Let,
                                    local: failed,
                                    declared: binding.span,
                                    given: None,
                                    known_case: None,
                                },
                                super::State::Assigned,
                            );
                        }
                        let checked = self.block(body, &arm.body);
                        otherwise.extend(checked.block);
                        body.scopes.pop();
                        body.forget_after(start);
                    }
                    body.states = Body::join_states(&before, &body.states);
                }
                None if ignored => {}
                None => {
                    let crash = failure(
                        self,
                        Diagnostic::new(
                            DiagnosticKind::CheckFailed,
                            call.span,
                            "`not_expected` failed, and nothing handled it",
                            "a failure nothing handles stops the program (D96)",
                        ),
                        &[When::Text("it carried ".into()), When::Operand(0)],
                    );
                    otherwise.push(checked::Statement::Check {
                        condition: checked::Expression { kind: Kind::Bool(false), machine_type: MachineType::Bool },
                        failure: crash,
                        operands: vec![read_now.clone()],
                    });
                }
            }
            block.push(checked::Statement::If { condition: same, then: vec![write], otherwise });
            Ok(())
        })())
    }

    /// The data `not_expected` carries, `val`, as a struct of its own for the arm that reads it (D360).
    fn not_expected_struct(&mut self, inner: &Type) -> usize {
        let id = self.program.structs.len();
        while self.aggregates.structs.len() < id {
            self.aggregates.structs.push(super::aggregates::StructInfo {
                name: String::new(),
                fields: Vec::new(),
                must_give: false,
                layout: super::aggregates::Layout::Written,
                sources: Vec::new(),
                facts: Vec::new(),
            });
        }
        self.aggregates.structs.push(super::aggregates::StructInfo {
            name: "not_expected".to_string(),
            fields: vec![super::aggregates::FieldInfo {
                name: "val".to_string(),
                value_type: inner.plain(),
                span: Span::default(),
            }],
            must_give: false,
            layout: super::aggregates::Layout::Written,
            sources: Vec::new(),
            facts: Vec::new(),
        });
        self.program.structs.push(vec![inner.machine()]);
        id
    }

    /// `std::coll::halves<T>` or `.mut`: a struct of two list views, `.first` and `.second`, made once per type (D382).
    pub fn halves_type(&mut self, item: &Type, changes: bool) -> Type {
        let name = format!("std::coll::halves{}<{item}>", if changes { ".mut" } else { "" });
        if let Some(found) = self.aggregates.names.get(&name) {
            return found.clone();
        }
        let view = if changes { super::types::ListView::Change } else { super::types::ListView::Read };
        let half = Type::List {
            item: Box::new(item.clone()),
            min: 0,
            max: None,
            len_type: super::types::IntegerKind::U64,
            grow: super::types::Grow::Double,
            view,
        };
        let id = self.program.structs.len();
        while self.aggregates.structs.len() < id {
            self.aggregates.structs.push(super::aggregates::StructInfo {
                name: String::new(),
                fields: Vec::new(),
                must_give: false,
                layout: super::aggregates::Layout::Written,
                sources: Vec::new(),
                facts: Vec::new(),
            });
        }
        let field = |name: &str| super::aggregates::FieldInfo {
            name: name.to_string(),
            value_type: half.clone(),
            span: Span::default(),
        };
        self.aggregates.structs.push(super::aggregates::StructInfo {
            name: name.clone(),
            fields: vec![field("first"), field("second")],
            must_give: false,
            layout: super::aggregates::Layout::Written,
            sources: Vec::new(),
            facts: Vec::new(),
        });
        self.program.structs.push(vec![half.machine(), half.machine()]);
        let made = Type::Struct { id, name: name.clone() };
        self.aggregates.names.insert(name, made.clone());
        made
    }

    /// `$xs.std::%%split.mut(at.items: 500)` or `$xs.std::%split(at.items: 500)`: two views of the list, which stays
    /// lent until the last use of either (D347, D358, D382).
    pub fn split(
        &mut self,
        body: &mut Body,
        expression: &syntax::Expression,
        receiver: &syntax::Expression,
        changes: bool,
        arguments: &[syntax::Argument],
    ) -> Check<Typed> {
        let Some(root) = root_name(receiver).cloned() else {
            return Err(self.not_yet(receiver.span, "splitting a fresh list".to_string()));
        };
        let Some(reached) = self.reach(body, receiver)? else { return Err(Reported) };
        let Type::List { item, view, .. } = reached.value_type.clone() else {
            return Err(self.not_yet(receiver.span, format!("splitting `{}`", reached.value_type)));
        };
        if view != super::types::ListView::Owned {
            return Err(self.not_yet(receiver.span, "splitting a view".to_string()));
        }
        if changes && !reached.root.mutable {
            return Err(self.error(Diagnostic::refused(
                receiver.span,
                format!("`{}` can't change, so it can't be lent with `Lend.mut`", root.text),
                "`let` declares a variable that can't change, and `let mut` one that can (D3)",
            )));
        }
        let [argument] = arguments else {
            return Err(self.not_yet(expression.span, "a split with these arguments".to_string()));
        };
        if argument.label.as_ref().map(|label| label.text.as_str()) != Some("at.items") {
            return Err(self.not_yet(argument.span, "a split without `at.items:`".to_string()));
        }
        let wide = Type::Integer { kind: super::types::IntegerKind::U64, bounds: None, unit: None };
        let at = self.value_into(body, &argument.value, &wide)?;
        let shown = self.slice(argument.value.span).to_string();
        let split_failure = failure(
            self,
            Diagnostic::new(
                DiagnosticKind::CheckFailed,
                expression.span,
                "this splits a list of \u{2} items at \u{1}",
                "a split's place is inside the list or at its end, like any position (D382, D133)",
            ),
            &[When::Text(format!("`at.items: {shown}`"))],
        );
        let path = self.any_place(body, receiver).map(|(_, path)| path).unwrap_or_default();
        let site = body.site(root.span);
        let place = crate::flow::Place { local: reached.root.local, path: path.clone(), shown: root.text.clone() };
        let spot = crate::flow::Spot { variable: root.span, place: receiver.span, site, marker: site };
        body.events.push(if changes {
            crate::flow::Event::LendMut { place, spot }
        } else {
            crate::flow::Event::Read { place, spot }
        });
        // The halves borrow the list for as long as either is used.
        self.pointer_borrow = Some((reached.root.local, path, changes));
        let halves = self.halves_type(&item, changes);
        Ok(Typed::new(
            halves,
            Kind::Split {
                list: checked::Place { local: reached.root.local, steps: reached.steps },
                at: Box::new(at),
                failure: Box::new(split_failure),
            },
        ))
    }
}
