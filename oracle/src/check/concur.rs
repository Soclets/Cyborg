//! Tasks from `@concur`: `@concur::spawn(on: Lend $runner)` before a call starts it as a task,
//! and `@concur::task.done(Give $t)` waits for it and gives back what it made.
//! (D343, D345, D346, D351, D356, D357, D375, D376, D377)
//!
//! A task is `must_Give`, so it's always ended before its block ends, and it keeps what its
//! call lent for as long as it exists, the way a stored borrow does, so two tasks can't both
//! change one value. Runners are `std::run::one_thread()` and `os::threads(count:)`. The oracle
//! runs every task on one thread, the way `std::run::one_thread` does: a task's body runs when
//! the code that spawned it first has to wait, in the order tasks were spawned. Atomics are
//! plain values here, since one task runs at a time, and a lock is never held by another task.
//! Channels aren't built yet.

use super::expression::RawCall;
use super::{Checker, ParameterInfo};
use crate::ast::{Argument, Expression, ExpressionKind, HandWord, ParameterWord};
use crate::ir::{self, AtomicOperation, Node, NodeKind, Overflow};
use crate::report::{Outcome, Report, ReportExtra, Span};
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
                    "the OS gives a pool's threads their stacks, so nothing is allocated here (D351, D384)",
                ));
            }
        }
        // `std::run::one_thread` switches tasks, so each has its own stack, from the allocator
        // the spawn names, with its size written. The oracle accepts any size. (D351, D352,
        // D380, D384)
        // trace: D351, D352, D380, D384
        if held.value_type == Type::Struct("std::run::one_thread".to_string()) {
            self.check_task_stack(&stack_options, span)?;
        }
        if let ExpressionKind::Try(inner) = &call.kind {
            return Err(Report::refused(
                call.span,
                "a task's failure comes back when it's done, not where it's spawned",
                "`@concur::task.done(Give $t)?` passes a task's failure up (D343, D357)",
            )
            .fix(self.text_of(inner.span).to_string()));
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

    /// A spawn's `alloc:` and `stack:`, which a runner that switches tasks needs: an allocator
    /// lent with `Lend`, and a literal size in `bytes`, `kib` or `mib`. (D351, D352, D370, D384)
    fn check_task_stack(&mut self, options: &[&Argument], span: Span) -> Outcome<()> {
        const WHY: &str = "each task on a runner that switches tasks has its own stack, from the allocator the spawn names, with its size written, as in `alloc: Lend $heap, stack: 4 kib` (D351, D352, D384)";
        let find = |label: &str| {
            options.iter().find(|option| {
                option
                    .label
                    .as_ref()
                    .is_some_and(|found| found.text == label)
            })
        };
        let (Some(alloc), Some(stack)) = (find("alloc"), find("stack")) else {
            return Err(Report::refused(
                span,
                "a spawn onto `std::run::one_thread` takes `alloc:` and `stack:`",
                WHY,
            ));
        };
        let mut handed = Vec::new();
        self.allocator_argument(&alloc.value, &mut handed)?;
        let is_size = matches!(
            &stack.value.kind,
            ExpressionKind::WithUnit { number, unit }
                if matches!(unit.text.as_str(), "bytes" | "kib" | "mib")
                    && super::expression::literal_integer(number).is_some_and(|size| size > 0)
        );
        if !is_size {
            return Err(Report::not_designed(
                stack.value.span,
                "a task's `stack:` that isn't a literal number of `bytes`, `kib` or `mib`",
                "",
            ));
        }
        Ok(())
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
            // trace: D354
            "atomic.new" => return self.check_atomic_new(arguments, span, path_span),
            // trace: D383
            "lock.new" => return self.check_lock_new(arguments, span, path_span),
            "lock.done" => return self.check_lock_done(arguments, span, path_span),
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

/// The orderings D354 names. The oracle runs one task at a time, where every ordering gives
/// the same answer.
const ORDERINGS: &[&str] = &["acquire", "relaxed"];

const WHY_ATOMIC: &str = "an atomic has `.%load()`, `.%add(val:)`, `.%sub(val:)`, `.%and.bit(val:)`, `.%or.bit(val:)`, `.%xor.bit(val:)`, `.%swap(val:)` and `.%compare_swap(expect:, val:)`, each with an ordering narrowing (D354, D360, D361)";

impl Checker<'_> {
    /// `@concur::atomic<u64>`: an integer or `bool` many tasks change without a lock. (D354)
    /// trace: D354
    pub(super) fn resolve_atomic_type(inner: Type, span: Span) -> Outcome<Type> {
        match &inner {
            Type::Integer {
                bound: None,
                unit: None,
                ..
            }
            | Type::Bool => Ok(Type::Atomic(Box::new(inner))),
            Type::Integer { .. } => Err(Report::not_designed(
                span,
                format!("an atomic `{inner}`, with a bound or a unit"),
                "",
            )),
            _ => Err(Report::refused(
                span,
                format!("an atomic holds an integer or a `bool`, and this is `{inner}`"),
                "an atomic is changed by the CPU's own indivisible instructions, which work on integers and `bool` (D354)",
            )),
        }
    }

    /// `@concur::atomic.new(0)`, with its type from where it goes. (D354)
    /// trace: D354
    pub(super) fn check_atomic_new(
        &mut self,
        arguments: &[Argument],
        span: Span,
        path_span: Span,
    ) -> Outcome<RawCall> {
        let target = self.context.call_expected.clone();
        let Some(Type::Atomic(inner)) = target.clone() else {
            return Err(Report::refused(
                path_span,
                "`@concur::atomic.new` goes where its type is written, as in `let $hits: @concur::atomic<u64>`",
                "every `let` writes its type (D277, D354)",
            ));
        };
        let [value] = arguments else {
            return Err(Report::refused(
                path_span,
                "`@concur::atomic.new` takes the value it starts with",
                "(D354)",
            ));
        };
        if value.label.is_some() {
            return Err(Report::refused(
                value.value.span,
                "`@concur::atomic.new` takes the value it starts with, with no label",
                "(D354)",
            ));
        }
        let node = self.check_into(&value.value, &inner)?;
        Ok(RawCall {
            node: Node {
                span,
                kind: node.kind,
            },
            value_type: Type::Atomic(inner),
            failures: Vec::new(),
            unnamed: None,
        })
    }

    /// An operation on an atomic, through `.%`, since it's shared with plain `Lend`. (D354, D360)
    /// trace: D354, D360, D361
    pub(super) fn check_atomic_method(
        &mut self,
        receiver: &Expression,
        inner: &Type,
        name: &crate::ast::Name,
        changes: bool,
        arguments: &[Argument],
        span: Span,
    ) -> Outcome<RawCall> {
        if changes {
            return Err(Report::refused(
                Span::new(name.span.start.saturating_sub(2), name.span.end),
                "an atomic changes through `.%`, not `.%%`",
                "an atomic is shared with plain `Lend` and changed by indivisible instructions, so its operations take `.%` (D354)",
            )
            .fix(format!("%{}", name.text)));
        }
        let (operation, ordering) = Self::atomic_operation(&name.text, inner, name.span)?;
        if let Some(ordering) = ordering {
            if !ORDERINGS.contains(&ordering.as_str()) {
                return Err(Report::not_designed(
                    name.span,
                    format!("the ordering `.{ordering}`"),
                    "",
                ));
            }
        }
        let wanted: &[&str] = match operation {
            AtomicOperation::Load => &[],
            AtomicOperation::CompareSwap => &["expect", "val"],
            _ => &["val"],
        };
        let labels: Vec<Option<&str>> = arguments
            .iter()
            .map(|argument| argument.label.as_ref().map(|label| label.text.as_str()))
            .collect();
        if labels != wanted.iter().map(|label| Some(*label)).collect::<Vec<_>>() {
            let shown: Vec<String> = wanted.iter().map(|label| format!("{label}:")).collect();
            return Err(Report::refused(
                span,
                if shown.is_empty() {
                    format!("`.%{}` takes nothing", name.text)
                } else {
                    format!("`.%{}` takes `{}`", name.text, shown.join(" "))
                },
                WHY_ATOMIC,
            ));
        }
        let held = self.check_held(receiver)?;
        let Some(place) = held.place else {
            return Err(Report::refused(
                receiver.span,
                "an atomic is changed where it lives, in a variable",
                WHY_ATOMIC,
            ));
        };
        let mut values = Vec::new();
        for argument in arguments {
            values.push(self.check_into(&argument.value, inner)?);
        }
        let (value_type, failures) = match operation {
            AtomicOperation::Load | AtomicOperation::Swap => (inner.clone(), Vec::new()),
            AtomicOperation::CompareSwap => (
                Type::Void,
                vec![super::Failure {
                    short: "not_expected".to_string(),
                    full: "not_expected".to_string(),
                    data: vec![("val".to_string(), inner.clone())],
                }],
            ),
            _ => (Type::Void, Vec::new()),
        };
        let kind = match inner {
            Type::Integer { kind, .. } => Some(*kind),
            _ => None,
        };
        Ok(RawCall {
            node: Node {
                span,
                kind: NodeKind::Atomic {
                    place,
                    operation,
                    kind,
                    values,
                },
            },
            value_type,
            failures,
            unnamed: None,
        })
    }

    /// The operation a method's name says, with any overflow narrowing, and the ordering
    /// narrowing that ends it, if one is written.
    fn atomic_operation(
        written: &str,
        inner: &Type,
        span: Span,
    ) -> Outcome<(AtomicOperation, Option<String>)> {
        let parts: Vec<&str> = written.split('.').collect();
        let (operation, rest) = match parts.as_slice() {
            ["and", "bit", rest @ ..] => (AtomicOperation::AndBit, rest),
            ["or", "bit", rest @ ..] => (AtomicOperation::OrBit, rest),
            ["xor", "bit", rest @ ..] => (AtomicOperation::XorBit, rest),
            ["load", rest @ ..] => (AtomicOperation::Load, rest),
            ["swap", rest @ ..] => (AtomicOperation::Swap, rest),
            ["compare_swap", rest @ ..] => (AtomicOperation::CompareSwap, rest),
            [word @ ("add" | "sub"), rest @ ..] => {
                let (overflow, rest) = match rest {
                    ["wrap", rest @ ..] => (Overflow::Wrap, rest),
                    ["saturate", rest @ ..] => (Overflow::Saturate, rest),
                    _ => (Overflow::Checked, rest),
                };
                let operation = if *word == "add" {
                    AtomicOperation::Add(overflow)
                } else {
                    AtomicOperation::Sub(overflow)
                };
                (operation, rest)
            }
            [word @ ("and" | "or" | "xor"), ..] => {
                return Err(Report::refused(
                    span,
                    format!("an atomic's bit operation is `.%{word}.bit`"),
                    "bit operators say `.bit`, so they can't be taken for logic (D361)",
                )
                .fix(format!("{word}.bit")));
            }
            _ => {
                return Err(Report::refused(
                    span,
                    format!("an atomic has no operation `.%{written}`"),
                    WHY_ATOMIC,
                ));
            }
        };
        let ordering = match rest {
            [] => None,
            [ordering] => Some((*ordering).to_string()),
            _ => {
                return Err(Report::not_designed(
                    span,
                    format!("the narrowings `.{}` together on an atomic", rest.join(".")),
                    "",
                ));
            }
        };
        if *inner == Type::Bool
            && !matches!(
                operation,
                AtomicOperation::Load | AtomicOperation::Swap | AtomicOperation::CompareSwap
            )
        {
            return Err(Report::not_designed(
                span,
                format!("`.%{written}` on an atomic `bool`"),
                "",
            ));
        }
        Ok((operation, ordering))
    }
}

impl Checker<'_> {
    /// `@concur::lock.new(0)`, with its type from where it goes; an owner goes in with `Give`.
    /// (D383)
    /// trace: D383
    pub(super) fn check_lock_new(
        &mut self,
        arguments: &[Argument],
        span: Span,
        path_span: Span,
    ) -> Outcome<RawCall> {
        let target = self.context.call_expected.clone();
        let Some(Type::Lock(inner)) = target else {
            return Err(Report::refused(
                path_span,
                "`@concur::lock.new` goes where its type is written, as in `let $hits: @concur::lock<i64>`",
                "every `let` writes its type (D277, D383)",
            ));
        };
        let [value] = arguments else {
            return Err(Report::refused(
                path_span,
                "`@concur::lock.new` takes the value it starts with",
                "(D383)",
            ));
        };
        let node = self.check_into(&value.value, &inner)?;
        Ok(RawCall {
            node: Node {
                span,
                kind: NodeKind::Struct(vec![(0, node)]),
            },
            value_type: Type::Lock(inner),
            failures: Vec::new(),
            unnamed: None,
        })
    }

    /// `@concur::lock.done(Give $l)` ends a lock and gives back what it held. (D383)
    /// trace: D383
    pub(super) fn check_lock_done(
        &mut self,
        arguments: &[Argument],
        span: Span,
        path_span: Span,
    ) -> Outcome<RawCall> {
        const WHY: &str = "`@concur::lock.done(Give $l)` ends a lock and gives back what it held, so it can be freed (D331, D383)";
        let [lock] = arguments else {
            return Err(Report::refused(
                path_span,
                "`@concur::lock.done` takes the lock it ends",
                WHY,
            ));
        };
        let Some(Type::Lock(inner)) = self.variable_type_of(&lock.value) else {
            return Err(Report::refused(
                lock.value.span,
                "`@concur::lock.done` ends a lock",
                WHY,
            ));
        };
        let parameter = ParameterInfo {
            name: "lock".to_string(),
            label: None,
            word: ParameterWord::Given,
            parameter_type: Type::Lock(inner.clone()),
            default: None,
            label_has_unit: false,
        };
        let mut handed = Vec::new();
        let node = match self.check_argument(&lock.value, &parameter, &mut handed)? {
            ir::Argument::Value(node) => node,
            ir::Argument::Alias(place) => Node {
                span: lock.value.span,
                kind: NodeKind::Read(place),
            },
        };
        Ok(RawCall {
            node: Node {
                span,
                kind: NodeKind::Field(Box::new(node), 0),
            },
            value_type: *inner,
            failures: Vec::new(),
            unnamed: None,
        })
    }

    /// `@concur::locked $from, $to as $a, $b:` takes every lock it names, in an order the locks
    /// set, and gives a `Lend.mut` of what each holds for the block, unlocking when it ends,
    /// however it ends. Locking inside it, and naming one lock twice, are refused, so deadlocks
    /// can't happen. With one task running at a time, nothing else ever holds a lock here.
    /// (D348, D349)
    /// trace: D348, D349
    pub(super) fn check_locked(
        &mut self,
        locks: &[Expression],
        names: &[crate::ast::Name],
        body: &[crate::ast::Statement],
        span: Span,
    ) -> Outcome<(Vec<ir::Instruction>, bool)> {
        self.require_limb("concur", span)?;
        if self.context.in_locked > 0 {
            return Err(Report::refused(
                span,
                "this locks inside a locked block",
                "a deadlock needs a task holding one lock while waiting for another, so locking inside a locked block is refused, and one block takes every lock it needs (D348, D349)",
            ));
        }
        if locks.len() != names.len() {
            return Err(Report::refused(
                span,
                format!(
                    "this takes {} locks and names {} values",
                    locks.len(),
                    names.len()
                ),
                "`@concur::locked $from, $to as $a, $b:` names what each lock holds, in order (D349)",
            ));
        }
        let mut seen: Vec<String> = Vec::new();
        let mut places = Vec::new();
        for lock in locks {
            let written = self.text_of(lock.span).to_string();
            if seen.contains(&written) {
                return Err(Report::refused(
                    lock.span,
                    format!("`{written}` is locked twice here"),
                    "a task taking one lock twice would wait on itself (D349)",
                ));
            }
            seen.push(written);
            let held = self.check_held(lock)?;
            let Type::Lock(inner) = held.value_type else {
                return Err(Report::refused(
                    lock.span,
                    format!("`{}` isn't a lock", held.value_type),
                    "`@concur::locked` takes locks, made with `@concur::lock.new` (D348, D383)",
                ));
            };
            let Some(mut place) = held.place else {
                return Err(Report::refused(
                    lock.span,
                    "a lock is locked where it lives, in a variable",
                    "(D348)",
                ));
            };
            place.path.push(ir::Step::Field(0));
            places.push((place, *inner));
        }
        self.context.scopes.push(std::collections::HashMap::new());
        let mut code = Vec::new();
        for ((place, inner), name) in places.into_iter().zip(names) {
            let slot = self.declare_variable(super::Variable {
                name: name.text.clone(),
                held_type: inner,
                mutable: true,
                origin: super::Origin::LentMutParameter,
                declared: name.span,
                must_give: false,
                constant: None,
                borrows: Vec::new(),
                borrowed_path: Vec::new(),
            });
            code.push(ir::Instruction {
                span: name.span,
                kind: ir::InstructionKind::LetAlias { slot, place },
            });
        }
        self.context.in_locked += 1;
        let result = self.check_statements(body);
        self.context.in_locked -= 1;
        let (body_code, terminated) = match result {
            Ok(done) => done,
            Err(report) => {
                self.context.scopes.pop();
                return Err(report);
            }
        };
        code.extend(body_code);
        self.end_held_borrows();
        let end_span = body.last().map_or(span, |last| last.span);
        let scope_check = if terminated {
            Ok(())
        } else {
            self.check_innermost_scope_end(end_span)
        };
        self.keep_promises();
        self.context.scopes.pop();
        scope_check?;
        Ok((code, terminated))
    }
}
