//! Tasks from `@concur`: `@concur::spawn(on: Lend $runner)` before a call starts it as a task,
//! and `@concur::task.done(Give $t)` waits for it and gives back what it made.
//! (D343, D345, D346, D351, D356, D357, D375, D376, D377)
//!
//! A task is `must_Give`, so it's always ended before its block ends, and it keeps what its
//! call lent for as long as it exists, the way a stored borrow does, so two tasks can't both
//! change one value. Runners are `std::run::one_thread()` and `os::threads(count:)`. The oracle
//! runs every task on one thread, the way `std::run::one_thread` does: a task's body runs when
//! the code that spawned it first has to wait, in the order tasks were spawned. Channels, locks
//! and atomics aren't built here yet.

use super::expression::RawCall;
use super::{Checker, ParameterInfo};
use crate::ast::{Argument, Expression, ExpressionKind, HandWord, ParameterWord};
use crate::ir::{self, Node, NodeKind};
use crate::report::{Outcome, Report, Span};
use crate::types::{Type, Value};

const WHY_DONE: &str = "`@concur::task.done(Give $left)` waits for a task, gives back what it made, and passes its failure up (D331, D357)";

const WHY_SPAWN: &str = "`@concur::spawn(on: Lend $runner)` goes before a call, which becomes a task on that runner (D345, D356)";

/// The runners the oracle knows, by their type's name.
fn is_runner(value_type: &Type) -> bool {
    matches!(value_type, Type::Struct(name) if name == "std::run::one_thread" || name == "os::threads" || name == "os::events")
}

impl Checker<'_> {
    /// `@concur::spawn(on: Lend $pool) sum(Lend $xs)`: the arguments are taken here, and only
    /// the body runs later, as a task. (D345, D356)
    /// trace: D345, D351, D356
    pub(super) fn check_spawn(
        &mut self,
        options: &[Argument],
        call: &Expression,
        span: Span,
    ) -> Outcome<(Node, Type)> {
        self.require_limb("concur", span)?;
        let mut runner = None;
        let mut stack_options = Vec::new();
        for option in options {
            match option.label.as_ref().map(|label| label.text.as_str()) {
                Some("on") if runner.is_none() => runner = Some(option),
                Some("alloc" | "stack") => stack_options.push(option),
                _ => {
                    return Err(Report::refused(
                        option.value.span,
                        "`@concur::spawn` takes `on:`, and on an event runner `alloc:` and `stack:`",
                        WHY_SPAWN,
                    ));
                }
            }
        }
        let Some(runner) = runner else {
            return Err(Report::refused(
                span,
                "`@concur::spawn` names its runner with `on:`",
                "every spawn names where its task runs, as every `append` names its allocator (D345)",
            ));
        };
        let ExpressionKind::Hand {
            word: HandWord::Lend,
            value: lent,
        } = &runner.value.kind
        else {
            return Err(Report::refused(
                runner.value.span,
                "a runner is shared with `Lend`",
                "many tasks run on one runner, so it's lent to each spawn (D345)",
            ));
        };
        let held = self.check_lent_place(lent, HandWord::Lend)?;
        if !is_runner(&held.value_type) {
            return Err(Report::refused(
                lent.span,
                format!("`{}` isn't a runner", held.value_type),
                "a task runs on `std::run::one_thread()`, `os::threads(count:)` or `os::events` (D345, D346, D377)",
            ));
        }
        if held.value_type == Type::Struct("os::events".to_string()) {
            return Err(Report::not_built(span, "a spawn onto `os::events`"));
        }
        if let Some(extra) = stack_options.first() {
            if held.value_type == Type::Struct("os::threads".to_string()) {
                return Err(Report::refused(
                    extra.value.span,
                    "a spawn onto `os::threads` takes neither `alloc:` nor `stack:`",
                    "the OS gives a pool's threads their stacks, so nothing is allocated here (D351)",
                ));
            }
            return Err(Report::not_designed(
                extra.value.span,
                "`alloc:` and `stack:` on a spawn onto `std::run::one_thread`",
                "",
            ));
        }
        if !matches!(call.kind, ExpressionKind::Call { .. }) {
            return Err(Report::refused(
                call.span,
                "`@concur::spawn` goes before a call",
                WHY_SPAWN,
            ));
        }
        let raw = self.check_raw_call(call)?;
        if !matches!(raw.node.kind, NodeKind::Call { .. }) {
            return Err(Report::not_built(
                call.span,
                "spawning a call to anything but a function of your own",
            ));
        }
        self.context.spawned_failures = Some(raw.failures);
        Ok((
            Node {
                span,
                kind: NodeKind::Spawn(Box::new(raw.node)),
            },
            Type::Task(Box::new(raw.value_type)),
        ))
    }

    /// What a spawn's call lends, which the task keeps for as long as it exists, and whether any
    /// of it is lent to change. (D347)
    /// trace: D347
    pub(super) fn spawn_borrows(&self, value: &Expression) -> (Vec<String>, bool) {
        let ExpressionKind::Spawn { call, .. } = &value.kind else {
            return (Vec::new(), false);
        };
        let ExpressionKind::Call { arguments, .. } = &call.kind else {
            return (Vec::new(), false);
        };
        let mut borrows = Vec::new();
        let mut changes = false;
        for argument in arguments {
            if let ExpressionKind::Hand {
                word: word @ (HandWord::Lend | HandWord::LendMut),
                value,
            } = &argument.value.kind
            {
                if let Some(root) = super::body::root_variable(value) {
                    for name in self.borrow_roots(&root) {
                        if !borrows.contains(&name) {
                            borrows.push(name);
                        }
                    }
                    changes = changes || *word == HandWord::LendMut;
                }
            }
        }
        (borrows, changes)
    }

    /// `@concur::task.done(Give $left)` waits for the task, gives back its result, passes its
    /// failure up, and ends its duty. Anything else from `@concur` isn't built yet. (D357)
    /// trace: D357
    pub(super) fn check_concur_call(
        &mut self,
        rest: &str,
        arguments: &[Argument],
        span: Span,
        path_span: Span,
    ) -> Outcome<RawCall> {
        self.require_limb("concur", path_span)?;
        match rest {
            "task.done" => {}
            "spawn" => {
                return Err(Report::refused(
                    path_span,
                    "`@concur::spawn` goes before a call",
                    WHY_SPAWN,
                ));
            }
            other => {
                return Err(Report::not_built(path_span, format!("`@concur::{other}`")));
            }
        }
        let [task] = arguments else {
            return Err(Report::refused(
                path_span,
                "`@concur::task.done` takes the task it ends",
                WHY_DONE,
            ));
        };
        let task_type = self.variable_type_of(&task.value);
        let Some(Type::Task(result)) = task_type.clone() else {
            return Err(Report::refused(
                task.value.span,
                "`@concur::task.done` ends a task",
                WHY_DONE,
            ));
        };
        let slot = match &task.value.kind {
            ExpressionKind::Hand { value, .. } => match &value.kind {
                ExpressionKind::Variable(name) => self.find_variable(name),
                _ => None,
            },
            _ => None,
        };
        let parameter = ParameterInfo {
            name: "task".to_string(),
            label: None,
            word: ParameterWord::Given,
            parameter_type: Type::Task(result.clone()),
            default: None,
            label_has_unit: false,
        };
        let mut handed = Vec::new();
        let node = match self.check_argument(&task.value, &parameter, &mut handed)? {
            ir::Argument::Value(node) => node,
            ir::Argument::Alias(place) => Node {
                span: task.value.span,
                kind: NodeKind::Read(place),
            },
        };
        // A task made here can fail with what its call can. One handed in from elsewhere came
        // through a type that doesn't carry failures.
        let local = slot
            .and_then(|slot| self.context.variables.get(slot))
            .is_some_and(|variable| variable.origin == super::Origin::Owned);
        let failures = match slot.and_then(|slot| self.context.task_failures.get(&slot).cloned()) {
            Some(failures) => failures,
            // Its `let` was refused, which has already said so.
            None if local => Vec::new(),
            None => {
                return Err(Report::not_designed(
                    task.value.span,
                    "what a task can fail with once it's handed to another function, since a function's type doesn't carry its failures",
                    "12.2",
                ));
            }
        };
        Ok(RawCall {
            node: Node {
                span,
                kind: NodeKind::TaskDone(Box::new(node)),
            },
            value_type: *result,
            failures,
            unnamed: None,
        })
    }

    /// `std::run::one_thread()`, the runner that works on any machine. (D376, D377)
    /// trace: D376, D377
    pub(super) fn check_one_thread_call(arguments: &[Argument], span: Span) -> Outcome<RawCall> {
        if let Some(extra) = arguments.first() {
            return Err(Report::refused(
                extra.value.span,
                "`std::run::one_thread()` takes nothing",
                "it runs every task on the one thread that's already running (D376, D377)",
            ));
        }
        Ok(RawCall {
            node: Node {
                span,
                kind: NodeKind::Constant(Value::Struct(Vec::new())),
            },
            value_type: Type::Struct("std::run::one_thread".to_string()),
            failures: Vec::new(),
            unnamed: None,
        })
    }

    /// `os::threads(count: 8)`, or `count: max` for a thread on every core. The count is always
    /// written. (D345, D346)
    /// trace: D345, D346
    pub(super) fn check_threads_call(
        &mut self,
        arguments: &[Argument],
        span: Span,
    ) -> Outcome<RawCall> {
        const WHY: &str = "the thread count is always written, a number or `max` for every core, so a program behaves the same everywhere unless it says otherwise (D345, D346)";
        let [count] = arguments else {
            return Err(Report::refused(span, "`os::threads` takes `count:`", WHY));
        };
        if count
            .label
            .as_ref()
            .is_none_or(|label| label.text != "count")
        {
            return Err(Report::refused(
                count.value.span,
                "`os::threads` takes `count:`",
                WHY,
            ));
        }
        let is_max = matches!(&count.value.kind, ExpressionKind::Path(path) if path.text == "max");
        if !is_max {
            let (_, found) = self.check_expression(
                &count.value,
                Some(&Type::plain_integer(crate::types::IntegerKind::U32)),
            )?;
            if !matches!(found, Type::Integer { unit: None, .. }) {
                return Err(Report::refused(
                    count.value.span,
                    format!("a thread count is a plain integer or `max`, and this is `{found}`"),
                    WHY,
                ));
            }
        }
        Ok(RawCall {
            node: Node {
                span,
                kind: NodeKind::Constant(Value::Struct(Vec::new())),
            },
            value_type: Type::Struct("os::threads".to_string()),
            failures: Vec::new(),
            unnamed: None,
        })
    }
}
