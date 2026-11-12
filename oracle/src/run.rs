//! Runs checked code, with every check on, as `Working` would. (D62)

use std::cell::RefCell;
use std::cmp::Ordering::{Equal, Greater, Less};
use std::io::Write;
use std::rc::Rc;

use crate::ast::{CompareOperator, Narrowing, WordOperator};
use crate::ir::Alias;
use crate::ir::{
    Argument, Arithmetic, Code, Contract, Function, Implementation, Instruction, InstructionKind,
    Node, NodeKind, Number, Place, Position, RangeCode, Reduction, Step, Table, Test,
};
use crate::report::{Kind, Outcome, Report, ReportExtra, Span};
use crate::types::{show_float, FloatKind, FloatRules, IntegerKind, Value};

/// How deep Cyborg calls can go here. Past this the oracle says so, instead of running out of
/// stack and crashing, which rule 6 forbids. What running out of stack means isn't decided.
const CALL_DEPTH_LIMIT: usize = 10_000;

/// Compile-time work stops here, as D89's "such as through a step limit" asks. The number itself
/// isn't chosen by anyone.
const COMPILE_TIME_STEP_LIMIT: u64 = 10_000_000;

/// A failure on its way up to something that handles it.
struct Raised {
    name: String,
    /// How `what:` says it, as in "`odd` failed" or "`$xs` is full".
    shown: String,
    when: Option<String>,
    /// What it carries, in the order declared, for `case negative $f:` to read.
    data: Vec<Value>,
    span: Span,
}

/// Why evaluating something stopped before giving a value.
enum Stop {
    Raised(Raised),
    Return(Value, Span),
    /// A borrow handed back, which still points at what it borrows. (D170)
    ReturnBorrow(Binding, Span),
    Break(usize),
    /// `instead 0;`, on its way to the call its arm belongs to.
    Instead(Value),
    Report(Box<Report>),
}

impl From<Box<Report>> for Stop {
    fn from(report: Box<Report>) -> Self {
        Stop::Report(report)
    }
}

/// A variable's home: its own value, or for a lent one, a place in its owner's value.
#[derive(Clone)]
struct Binding {
    root: Rc<RefCell<Value>>,
    path: Vec<usize>,
}

impl Binding {
    fn fresh(value: Value) -> Self {
        Self {
            root: Rc::new(RefCell::new(value)),
            path: Vec::new(),
        }
    }
}

/// Turns a position into an item's index, or stops the program: a bad position is a bounds check.
/// trace: D26, D133
fn item_index(number: i128, count: usize, position: &Position) -> Result<usize, Stop> {
    let index = if position.counts_from_zero {
        number
    } else {
        number - 1
    };
    usize::try_from(index)
        .ok()
        .filter(|index| *index < count)
        .ok_or_else(|| {
            let word = if position.counts_from_zero {
                "offset.items"
            } else {
                "pos"
            };
            Stop::Report(
                Report::check_failed(
                    position.span,
                    format!("`{word}: {number}` is outside `{}`", position.shown),
                    "a bad position is a bounds check (D26, D133)",
                )
                .when(format!("it holds {count} items")),
            )
        })
}

fn walk<'v>(value: &'v Value, path: &[usize]) -> Option<&'v Value> {
    let mut current = value;
    for step in path {
        current = match current {
            Value::Struct(fields) | Value::Case(_, fields) | Value::List(fields, _) => {
                fields.get(*step)?
            }
            _ => return None,
        };
    }
    Some(current)
}

fn walk_mut<'v>(value: &'v mut Value, path: &[usize]) -> Option<&'v mut Value> {
    let mut current = value;
    for step in path {
        current = match current {
            Value::Struct(fields) | Value::Case(_, fields) | Value::List(fields, _) => {
                fields.get_mut(*step)?
            }
            _ => return None,
        };
    }
    Some(current)
}

struct Frame {
    slots: Vec<Option<Binding>>,
    /// For a generic, how the types filling its type parameters do each operation. (D235)
    tables: Vec<Table>,
}

struct Runner<'c> {
    code: &'c Code,
    globals: Vec<Value>,
    depth: usize,
    steps: u64,
    step_limit: Option<u64>,
    out: std::io::BufWriter<std::io::Stdout>,
    /// The terminal writers the program made, by number. (D273)
    writers: Vec<Writer>,
    /// What each pointer the program made points at, by number. (D315)
    pointers: Vec<PointerTarget>,
}

/// A pointer's target: a variable from `.of`, or an address from `.at`. (D315)
enum PointerTarget {
    Variable(Binding),
    Address,
}

/// A terminal writer: which stream, and for a buffered one, its size and what it holds.
/// trace: D273
struct Writer {
    stderr: bool,
    size: Option<usize>,
    held: Vec<u8>,
}

/// trace: D89
fn step_limit_report(span: Span) -> Box<Report> {
    Report::refused(
        span,
        "compile-time work didn't finish within its step limit",
        "compile-time code always finishes, through a step limit that refuses it (D89, D91)",
    )
}

fn oracle_bug(span: Span, what: &str) -> Stop {
    Stop::Report(Report::check_failed(
        span,
        format!("the oracle got lost: {what}"),
        "this is a bug in the oracle, not in your program, so please report it with this message and the program (rule 6)",
    ))
}

/// Works out what `bake` and `const` need while building, and refuses if that can't finish. (D91)
pub fn check_compile_time(code: &Code) -> Outcome<()> {
    let mut runner = Runner::new(code, Some(COMPILE_TIME_STEP_LIMIT));
    let mut frame = Frame {
        slots: Vec::new(),
        tables: Vec::new(),
    };
    let mut globals_done = 0;
    for node in &code.compile_time {
        let result = runner.evaluate(node, &mut frame);
        let value = match result {
            Ok(value) => value,
            Err(Stop::Report(mut report)) => {
                if report.kind == Kind::CheckFailed {
                    report.kind = Kind::Refused;
                }
                return Err(report);
            }
            Err(Stop::Raised(raised)) => {
                return Err(Report::refused(
                    node.span,
                    format!("{}, while working this out at compile time", raised.shown),
                    "a `const`'s value must be fully known at compile time (D83, D91)",
                ));
            }
            Err(Stop::Break(_) | Stop::Instead(_)) => {
                return Err(Report::refused(
                    node.span,
                    "this `break` jumps out of compile-time code",
                    "a `const`'s value must be fully known at compile time (D83, D91)",
                ));
            }
            Err(Stop::Return(_, span) | Stop::ReturnBorrow(_, span)) => {
                return Err(Report::refused(
                    span,
                    "this `return` jumps out of compile-time code",
                    "a `const`'s value must be fully known at compile time (D83, D91)",
                ));
            }
        };
        // Globals come first in the list, in order, so later work can read them.
        if globals_done < code.globals.len() {
            runner.globals.push(value);
            globals_done += 1;
        }
    }
    Ok(())
}

/// Runs the program from `fn.main`.
pub fn run(code: &Code) -> Outcome<()> {
    let mut runner = Runner::new(code, None);
    let mut frame = Frame {
        slots: Vec::new(),
        tables: Vec::new(),
    };
    for node in &code.globals {
        match runner.evaluate(node, &mut frame) {
            Ok(value) => runner.globals.push(value),
            Err(Stop::Report(report)) => return Err(report),
            Err(_) => return Ok(()),
        }
    }
    let result = runner.call(code.main, Vec::new(), Vec::new(), Span::new(0, 0));
    // A crash flushes every buffered writer still holding output before its message, so the
    // last thing printed before a crash always shows. (D274)
    // trace: D274
    let crashed_at = match &result {
        Err(Stop::Report(report)) if report.kind == Kind::CheckFailed => Some(report.span),
        Err(Stop::Raised(raised)) => Some(raised.span),
        _ => None,
    };
    let crash_flushed = crashed_at.map(|span| runner.flush_every_writer(span));
    let flushed = runner.out.flush();
    let crash_note = |report: Box<Report>| match (crashed_at, crash_flushed) {
        (Some(span), Some(false)) => report.because(
            span,
            "and a buffered writer's output couldn't be written as the program crashed",
        ),
        _ => report,
    };
    match result {
        Ok(_) => flushed.map_err(|error| {
            Report::check_failed(
                Span::new(0, 0),
                format!("the program's output couldn't be written: {error}"),
                "a failure nothing handles stops the program (D96)",
            )
        }),
        Err(Stop::Report(report)) => Err(crash_note(report)),
        Err(Stop::Raised(raised)) => Err(crash_note(Report::check_failed(
            raised.span,
            format!("{}, and nothing handled it", raised.shown),
            "a failure nothing handles stops the program (D96)",
        ))),
        Err(Stop::Return(..) | Stop::ReturnBorrow(..) | Stop::Break(_) | Stop::Instead(_)) => {
            Ok(())
        }
    }
}

impl<'c> Runner<'c> {
    fn new(code: &'c Code, step_limit: Option<u64>) -> Self {
        Self {
            code,
            globals: Vec::new(),
            depth: 0,
            steps: 0,
            step_limit,
            out: std::io::BufWriter::new(std::io::stdout()),
            writers: Vec::new(),
            pointers: Vec::new(),
        }
    }

    fn call(
        &mut self,
        function: usize,
        arguments: Vec<(usize, Binding)>,
        tables: Vec<Table>,
        call_span: Span,
    ) -> Result<Binding, Stop> {
        let code = self.code;
        let Some(function) = code.functions.get(function) else {
            return Err(oracle_bug(
                call_span,
                "a call to a function that isn't there",
            ));
        };
        self.depth += 1;
        if self.depth > CALL_DEPTH_LIMIT {
            self.depth -= 1;
            if self.step_limit.is_some() {
                // trace: D89
                return Err(Stop::Report(step_limit_report(call_span)));
            }
            return Err(Stop::Report(Report::not_designed(
                call_span,
                format!("calls deeper than the oracle's limit of {CALL_DEPTH_LIMIT}, and what running out of stack means"),
                "",
            )));
        }
        let mut frame = Frame {
            slots: vec![None; function.slot_count],
            tables,
        };
        for (parameter, binding) in arguments {
            if let Some(slot) = frame.slots.get_mut(parameter) {
                *slot = Some(binding);
            }
        }
        let result = self.call_body(function, &mut frame, call_span);
        self.depth -= 1;
        result
    }

    fn call_body(
        &mut self,
        function: &Function,
        frame: &mut Frame,
        call_span: Span,
    ) -> Result<Binding, Stop> {
        for contract in &function.requires {
            // trace: D29
            self.check_contract(contract, frame, "requires", call_span, "called here")?;
        }
        let returned = match self.run_block(&function.body, frame) {
            Ok(()) => {
                return Err(oracle_bug(
                    call_span,
                    "a function ran off its end, which the checker refuses",
                ));
            }
            Err(Stop::Return(value, span)) => (Binding::fresh(value), span),
            // trace: D170
            Err(Stop::ReturnBorrow(binding, span)) => (binding, span),
            Err(other) => return Err(other),
        };
        if !function.ensures.is_empty() {
            if let Some(slot) = frame.slots.get_mut(function.result_slot) {
                *slot = Some(returned.0.clone());
            }
            for contract in &function.ensures {
                self.check_contract(
                    contract,
                    frame,
                    "ensures",
                    returned.1,
                    "this `return` led here",
                )?;
            }
        }
        Ok(returned.0)
    }

    fn check_contract(
        &mut self,
        contract: &Contract,
        frame: &mut Frame,
        word: &str,
        because: Span,
        note: &str,
    ) -> Result<(), Stop> {
        let value = self.evaluate(&contract.condition, frame)?;
        if value == Value::Bool(true) {
            return Ok(());
        }
        let mut shown = Vec::new();
        for (name, node) in &contract.shown {
            let value = self.evaluate(node, frame)?;
            if let Some(text) = value.shown() {
                shown.push(format!("`{name}` is {text}"));
            }
        }
        let mut report = Report::check_failed(
            contract.condition.span,
            format!("this `{word}` is false"),
            "contracts are checked at runtime, and a false one stops the program (D29)",
        );
        if !shown.is_empty() {
            report = report.when(shown.join(", "));
        }
        Err(Stop::Report(report.because(because, note)))
    }

    /// Runs a block, then its `defer`s in reverse order, however it ended. (D325)
    /// trace: D325
    fn run_block(&mut self, body: &[Instruction], frame: &mut Frame) -> Result<(), Stop> {
        let mut deferred: Vec<&Instruction> = Vec::new();
        let mut result = Ok(());
        for instruction in body {
            if let InstructionKind::Defer(inner) = &instruction.kind {
                deferred.push(inner);
                continue;
            }
            result = self.run_instruction(instruction, frame);
            if result.is_err() {
                break;
            }
        }
        for inner in deferred.into_iter().rev() {
            let ran = self.run_instruction(inner, frame);
            if result.is_ok() {
                result = ran;
            }
        }
        result
    }

    /// Works out where a place is right now. An item's position runs here, and is checked here.
    /// Runs a call, keeping what it hands back as a binding, since a borrow still points at what
    /// it borrows. (D170)
    fn call_binding(&mut self, node: &Node, frame: &mut Frame) -> Result<Binding, Stop> {
        let NodeKind::Call {
            function,
            arguments,
            tables,
        } = &node.kind
        else {
            return Err(oracle_bug(
                node.span,
                "a borrow from something that isn't a call",
            ));
        };
        let mut bindings = Vec::with_capacity(arguments.len());
        for (parameter, argument) in arguments {
            let binding = match argument {
                Argument::Value(value) => Binding::fresh(self.evaluate(value, frame)?),
                Argument::Alias(place) => self.binding_of(frame, place, node.span)?,
            };
            bindings.push((*parameter, binding));
        }
        let tables = Self::resolve_tables(tables, frame, node.span)?;
        self.call(*function, bindings, tables, node.span)
    }

    fn binding_of(
        &mut self,
        frame: &mut Frame,
        place: &Place,
        span: Span,
    ) -> Result<Binding, Stop> {
        let mut binding = frame
            .slots
            .get(place.slot)
            .and_then(Option::as_ref)
            .cloned()
            .ok_or_else(|| oracle_bug(span, "a variable was read before it was set"))?;
        for step in &place.path {
            match step {
                Step::Field(index) => binding.path.push(*index),
                Step::Item(position) => {
                    let number = self.position_number(position, frame)?;
                    let count = {
                        let root = binding.root.borrow();
                        match walk(&root, &binding.path) {
                            Some(Value::List(items, _)) => items.len(),
                            _ => return Err(oracle_bug(span, "an item of something else")),
                        }
                    };
                    binding.path.push(item_index(number, count, position)?);
                }
            }
        }
        Ok(binding)
    }

    fn position_number(&mut self, position: &Position, frame: &mut Frame) -> Result<i128, Stop> {
        match self.evaluate(&position.value, frame)? {
            Value::Integer(number) => Ok(number),
            _ => Err(oracle_bug(position.span, "a position that isn't a number")),
        }
    }

    fn read(&mut self, frame: &mut Frame, place: &Place, span: Span) -> Result<Value, Stop> {
        let binding = self.binding_of(frame, place, span)?;
        let root = binding.root.borrow();
        walk(&root, &binding.path)
            .cloned()
            .ok_or_else(|| oracle_bug(span, "a field wasn't where the checker said"))
    }

    fn set_slot(frame: &mut Frame, slot: usize, binding: Binding, span: Span) -> Result<(), Stop> {
        let Some(found) = frame.slots.get_mut(slot) else {
            return Err(oracle_bug(span, "a variable has no slot"));
        };
        *found = Some(binding);
        Ok(())
    }

    #[allow(clippy::too_many_lines)] // One arm per instruction, read top to bottom.
    fn run_instruction(
        &mut self,
        instruction: &Instruction,
        frame: &mut Frame,
    ) -> Result<(), Stop> {
        let span = instruction.span;
        self.step(span)?;
        match &instruction.kind {
            // A `defer` outside a block's own list, which `run_block` keeps, runs at once.
            InstructionKind::Defer(inner) => self.run_instruction(inner, frame),
            InstructionKind::Let { slot, value } => {
                let value = self.evaluate(value, frame)?;
                Self::set_slot(frame, *slot, Binding::fresh(value), span)
            }
            InstructionKind::LetAlias { slot, place } => {
                let binding = self.binding_of(frame, place, span)?;
                Self::set_slot(frame, *slot, binding, span)
            }
            // trace: D170
            InstructionKind::LetCallAlias { slot, call } => {
                let binding = self.call_binding(call, frame)?;
                Self::set_slot(frame, *slot, binding, span)
            }
            InstructionKind::ReturnBorrow(alias) => {
                let binding = match alias {
                    Alias::Place(place) => self.binding_of(frame, place, span)?,
                    Alias::Call(call) => self.call_binding(call, frame)?,
                };
                Err(Stop::ReturnBorrow(binding, span))
            }
            InstructionKind::Set { place, value } => {
                // Left to right as written, so an item's position runs before the value does.
                let binding = self.binding_of(frame, place, span)?;
                let value = self.evaluate(value, frame)?;
                let mut root = binding.root.borrow_mut();
                let Some(target) = walk_mut(&mut root, &binding.path) else {
                    return Err(oracle_bug(span, "a field wasn't where the checker said"));
                };
                *target = value;
                Ok(())
            }
            // trace: D126
            InstructionKind::If { links, otherwise } => {
                for (test, body) in links {
                    if self.test_holds(test, frame, span)? {
                        return self.run_block(body, frame);
                    }
                }
                self.run_block(otherwise, frame)
            }
            InstructionKind::Match { subject, arms } => {
                let Value::Case(found, fields) = self.evaluate(subject, frame)? else {
                    return Err(oracle_bug(span, "`match` on something that isn't a union"));
                };
                for arm in arms {
                    if arm.case == found {
                        // trace: D139
                        if let Some(slot) = arm.slot {
                            let named = Binding::fresh(Value::Case(found, fields));
                            Self::set_slot(frame, slot, named, span)?;
                        }
                        return self.run_block(&arm.body, frame);
                    }
                }
                Err(oracle_bug(span, "a `match` missed a case the checker saw"))
            }
            InstructionKind::For {
                loop_id,
                slot,
                collection,
                body,
            } => {
                let Value::List(items, _) = self.evaluate(collection, frame)? else {
                    return Err(oracle_bug(
                        span,
                        "`for` on something that isn't a collection",
                    ));
                };
                for item in items {
                    Self::set_slot(frame, *slot, Binding::fresh(item), span)?;
                    if self.run_loop_body(*loop_id, body, frame, span)? {
                        break;
                    }
                }
                Ok(())
            }
            // Each round's name is the item itself, so a change goes into the list. (D180)
            InstructionKind::ForChunks(chunks) => self.run_chunks(chunks, frame, span),
            InstructionKind::ForInPlace {
                loop_id,
                slot,
                place,
                body,
            } => {
                let base = self.binding_of(frame, place, span)?;
                let count = match walk(&base.root.borrow(), &base.path) {
                    Some(Value::List(items, _)) => items.len(),
                    _ => {
                        return Err(oracle_bug(
                            span,
                            "`for` on something that isn't a collection",
                        ))
                    }
                };
                for index in 0..count {
                    let mut path = base.path.clone();
                    path.push(index);
                    let item = Binding {
                        root: Rc::clone(&base.root),
                        path,
                    };
                    Self::set_slot(frame, *slot, item, span)?;
                    if self.run_loop_body(*loop_id, body, frame, span)? {
                        break;
                    }
                }
                Ok(())
            }
            InstructionKind::ForRange {
                loop_id,
                slot,
                range,
                body,
            } => self.run_range(*loop_id, *slot, range, body, frame),
            InstructionKind::While {
                loop_id,
                condition,
                body,
            } => {
                while self.evaluate(condition, frame)? == Value::Bool(true) {
                    if self.run_loop_body(*loop_id, body, frame, span)? {
                        break;
                    }
                }
                Ok(())
            }
            InstructionKind::Forever { loop_id, body } => {
                while !self.run_loop_body(*loop_id, body, frame, span)? {}
                Ok(())
            }
            InstructionKind::Break(loop_id) => Err(Stop::Break(*loop_id)),
            InstructionKind::Instead(value) => {
                let value = self.evaluate(value, frame)?;
                Err(Stop::Instead(value))
            }
            InstructionKind::Return(value) => {
                let value = self.evaluate(value, frame)?;
                Err(Stop::Return(value, span))
            }
            InstructionKind::Fail { name, shown, data } => {
                let mut when = Vec::new();
                let mut values = Vec::new();
                for node in data {
                    let value = self.evaluate(node, frame)?;
                    if let Some(text) = value.shown() {
                        when.push(text);
                    }
                    values.push(value);
                }
                Err(Stop::Raised(Raised {
                    name: name.clone(),
                    shown: format!("`{shown}` failed"),
                    when: (!when.is_empty()).then(|| format!("it carried {}", when.join(", "))),
                    data: values,
                    span,
                }))
            }
            InstructionKind::Evaluate(value) => {
                self.evaluate(value, frame)?;
                Ok(())
            }
            InstructionKind::Block(body) => self.run_block(body, frame),
        }
    }

    /// Runs one time round a loop, and says whether a `break` stopped it. Each time counts as a
    /// step, so an empty `forever` in compile-time code still reaches the limit.
    fn run_loop_body(
        &mut self,
        loop_id: usize,
        body: &[Instruction],
        frame: &mut Frame,
        span: Span,
    ) -> Result<bool, Stop> {
        self.step(span)?;
        match self.run_block(body, frame) {
            Ok(()) => Ok(false),
            Err(Stop::Break(target)) if target == loop_id => Ok(true),
            Err(other) => Err(other),
        }
    }

    /// trace: D128
    fn run_range(
        &mut self,
        loop_id: usize,
        slot: usize,
        range: &RangeCode,
        body: &[Instruction],
        frame: &mut Frame,
    ) -> Result<(), Stop> {
        let RangeCode {
            from,
            end,
            included,
            step,
        } = range;
        let included = *included;
        let mut current = self.range_number(from, frame)?;
        let last = self.range_number(end, frame)?;
        let by = match step {
            Some(node) => self.range_number(node, frame)?,
            None => 1,
        };
        // A compiler skips position checks the range already proves. The oracle stays plain and runs
        // them, and they pass, so the two look the same from outside.
        // trace: D147
        // A range that can't reach its end runs zero rounds, which the loop below does on its own.
        // A step of 0 would never finish, so that's a failure, and `for` has nothing to handle it.
        // trace: D145
        if by == 0 {
            return Err(Stop::Report(
                Report::check_failed(
                    step.as_ref().map_or(from.span, |node| node.span),
                    "`step_by:` is 0, and nothing handled it",
                    "a step of 0 is a failure, since it never ends (D145, D96)",
                )
                .when(format!("from {current}, ending at {last}")),
            ));
        }
        loop {
            let inside = match (by > 0, included) {
                (true, true) => current <= last,
                (true, false) => current < last,
                (false, true) => current >= last,
                (false, false) => current > last,
            };
            if !inside {
                return Ok(());
            }
            Self::set_slot(
                frame,
                slot,
                Binding::fresh(Value::Integer(current)),
                from.span,
            )?;
            if self.run_loop_body(loop_id, body, frame, from.span)? {
                return Ok(());
            }
            current += by;
        }
    }

    fn range_number(&mut self, node: &Node, frame: &mut Frame) -> Result<i128, Stop> {
        match self.evaluate(node, frame)? {
            Value::Integer(number) => Ok(number),
            _ => Err(oracle_bug(node.span, "a range of something else")),
        }
    }

    fn test_holds(&mut self, test: &Test, frame: &mut Frame, span: Span) -> Result<bool, Stop> {
        match test {
            Test::Condition(condition) => Ok(self.evaluate(condition, frame)? == Value::Bool(true)),
            // trace: D20
            Test::Range { subject, low, high } => match self.evaluate(subject, frame)? {
                Value::Integer(value) => Ok(*low <= value && value <= *high),
                _ => Err(oracle_bug(span, "the test for a bound on something else")),
            },
            // trace: D158, D20
            Test::FloatRange { subject, low, high } => {
                let value = self.evaluate(subject, frame)?;
                let Some(number) = float_value(&value) else {
                    return Err(oracle_bug(span, "the test for a bound on something else"));
                };
                Ok(float_in_bound(number, *low, *high))
            }
            // trace: D9
            Test::Case { subject, case } => match self.evaluate(subject, frame)? {
                Value::Case(found, _) => Ok(found == *case),
                _ => Err(oracle_bug(span, "the test for a case on something else")),
            },
        }
    }

    fn step(&mut self, span: Span) -> Result<(), Stop> {
        if let Some(limit) = self.step_limit {
            self.steps += 1;
            if self.steps > limit {
                return Err(Stop::Report(step_limit_report(span)));
            }
        }
        Ok(())
    }

    fn evaluate_all(
        &mut self,
        fields: &[(usize, Node)],
        frame: &mut Frame,
    ) -> Result<Vec<Value>, Stop> {
        let mut values: Vec<(usize, Value)> = Vec::new();
        for (index, node) in fields {
            values.push((*index, self.evaluate(node, frame)?));
        }
        values.sort_by_key(|(index, _)| *index);
        Ok(values.into_iter().map(|(_, value)| value).collect())
    }

    #[allow(clippy::too_many_lines)] // One arm per form of value, read top to bottom.
    fn evaluate(&mut self, node: &Node, frame: &mut Frame) -> Result<Value, Stop> {
        let span = node.span;
        self.step(span)?;
        match &node.kind {
            NodeKind::Constant(value) => Ok(value.clone()),
            NodeKind::Void => Ok(Value::Void),
            NodeKind::Read(place) => self.read(frame, place, span),
            NodeKind::Item(base, position) => {
                let value = self.evaluate(base, frame)?;
                let number = self.position_number(position, frame)?;
                let Value::List(items, _) = value else {
                    return Err(oracle_bug(span, "an item of something else"));
                };
                let index = item_index(number, items.len(), position)?;
                items
                    .into_iter()
                    .nth(index)
                    .ok_or_else(|| oracle_bug(span, "an item wasn't where the check said"))
            }
            NodeKind::Global(index) => self
                .globals
                .get(*index)
                .cloned()
                .ok_or_else(|| oracle_bug(span, "a constant read before it was worked out")),
            NodeKind::Field(base, index) => {
                let value = self.evaluate(base, frame)?;
                walk(&value, &[*index])
                    .cloned()
                    .ok_or_else(|| oracle_bug(span, "a field wasn't where the checker said"))
            }
            NodeKind::Struct(fields) => Ok(Value::Struct(self.evaluate_all(fields, frame)?)),
            NodeKind::Case(case, fields) => {
                Ok(Value::Case(*case, self.evaluate_all(fields, frame)?))
            }
            NodeKind::Items(items) => {
                let mut values = Vec::with_capacity(items.len());
                for item in items {
                    values.push(self.evaluate(item, frame)?);
                }
                Ok(Value::List(values, None))
            }
            NodeKind::All(value, count) => {
                let value = self.evaluate(value, frame)?;
                Ok(Value::List(vec![value; *count], None))
            }
            NodeKind::Arithmetic {
                operator,
                number,
                left,
                right,
            } => {
                let left = self.evaluate(left, frame)?;
                let right = self.evaluate(right, frame)?;
                arithmetic(*operator, *number, &left, &right, span).map_err(Stop::Report)
            }
            NodeKind::BitNot(kind, value) => {
                let Value::Integer(value) = self.evaluate(value, frame)? else {
                    return Err(oracle_bug(span, "`bit_not` on something else"));
                };
                Ok(Value::Integer(wrap(!value, *kind)))
            }
            NodeKind::And(left, right) => {
                if self.evaluate(left, frame)? != Value::Bool(true) {
                    return Ok(Value::Bool(false));
                }
                self.evaluate(right, frame)
            }
            NodeKind::Or(left, right) => {
                if self.evaluate(left, frame)? == Value::Bool(true) {
                    return Ok(Value::Bool(true));
                }
                self.evaluate(right, frame)
            }
            NodeKind::AndBoth(left, right) => {
                let left = self.evaluate(left, frame)?;
                let right = self.evaluate(right, frame)?;
                Ok(Value::Bool(
                    left == Value::Bool(true) && right == Value::Bool(true),
                ))
            }
            NodeKind::OrBoth(left, right) => {
                let left = self.evaluate(left, frame)?;
                let right = self.evaluate(right, frame)?;
                Ok(Value::Bool(
                    left == Value::Bool(true) || right == Value::Bool(true),
                ))
            }
            NodeKind::Negate(number, value) => {
                let value = self.evaluate(value, frame)?;
                negate(*number, &value, span).map_err(Stop::Report)
            }
            NodeKind::Not(value) => {
                let value = self.evaluate(value, frame)?;
                Ok(Value::Bool(value != Value::Bool(true)))
            }
            NodeKind::Compare { first, rest } => {
                let mut left = self.evaluate(first, frame)?;
                for (operator, right) in rest {
                    // trace: D124
                    let right = self.evaluate(right, frame)?;
                    if !compare(*operator, &left, &right, span)? {
                        return Ok(Value::Bool(false));
                    }
                    left = right;
                }
                Ok(Value::Bool(true))
            }
            // trace: D256
            NodeKind::Maths { function, value } => {
                let Value::F64(x) = self.evaluate(value, frame)? else {
                    return Err(oracle_bug(
                        span,
                        "`std::math` of something that isn't an `f64`",
                    ));
                };
                crate::maths::correctly_rounded(*function, x)
                    .map(|(found, _)| Value::F64(found))
                    .map_err(|problem| oracle_bug(span, &problem))
            }
            NodeKind::Decibels {
                value,
                number,
                reference,
                to_amount,
            } => {
                let value = self.evaluate(value, frame)?;
                decibels(&value, *number, *reference, *to_amount, span)
            }
            NodeKind::Generic {
                parameter,
                operator,
                left,
                right,
            } => {
                let left = self.evaluate(left, frame)?;
                let right = self.evaluate(right, frame)?;
                self.generic(*parameter, operator, left, right, frame, span)
            }
            NodeKind::Call { .. } => {
                let binding = self.call_binding(node, frame)?;
                let root = binding.root.borrow();
                walk(&root, &binding.path)
                    .cloned()
                    .ok_or_else(|| oracle_bug(span, "a call handed back a borrow of nothing"))
            }
            NodeKind::FormatPrint {
                writer,
                pieces,
                values,
                stored,
            } => self.format_print(writer, pieces, values, *stored, frame, span),
            // trace: D273
            NodeKind::Writer { stderr, size } => {
                self.writers.push(Writer {
                    stderr: *stderr,
                    size: *size,
                    held: Vec::new(),
                });
                let number = i128::try_from(self.writers.len() - 1).unwrap_or(i128::MAX);
                Ok(Value::Struct(vec![Value::Integer(number)]))
            }
            NodeKind::PointerOf(_)
            | NodeKind::PointerAt(_)
            | NodeKind::PointerRead(_)
            | NodeKind::PointerWrite { .. }
            | NodeKind::PointerMoved { .. } => self.evaluate_pointer(node, frame),
            NodeKind::Lanewise { .. }
            | NodeKind::LanewiseCompare { .. }
            | NodeKind::Reduce { .. }
            | NodeKind::LanePick { .. }
            | NodeKind::Shuffle { .. }
            | NodeKind::JoinLanes { .. }
            | NodeKind::Lookup { .. } => self.evaluate_simd(node, frame),
            // Both sides run before one is chosen. (D340, D302)
            // trace: D340
            NodeKind::Pick {
                condition,
                if_true,
                if_false,
            } => {
                let condition = self.evaluate(condition, frame)?;
                let if_true = self.evaluate(if_true, frame)?;
                let if_false = self.evaluate(if_false, frame)?;
                Ok(if condition == Value::Bool(true) {
                    if_true
                } else {
                    if_false
                })
            }
            // trace: D326, D328
            NodeKind::FitText { value, bytes } => {
                let value = self.evaluate(value, frame)?;
                let Value::Str(text) = &value else {
                    return Err(oracle_bug(span, "text that isn't text"));
                };
                let length = i128::try_from(text.len()).unwrap_or(i128::MAX);
                if length > *bytes {
                    return Err(Stop::Raised(Raised {
                        name: String::new(),
                        shown: format!("this text doesn't fit `str[len.max: {bytes} bytes]`"),
                        when: Some(format!("it's {length} bytes")),
                        data: Vec::new(),
                        span,
                    }));
                }
                Ok(value)
            }
            // trace: D328, D331
            NodeKind::TextFrom { text, allocator } => {
                self.allocator_number(allocator, frame)?;
                self.evaluate(text, frame)
            }
            NodeKind::TextDone { text, allocator } => {
                self.evaluate(text, frame)?;
                self.allocator_number(allocator, frame)?;
                Ok(Value::Void)
            }
            // Every `os::heap()` is a handle to the one OS heap, so they share its number, 0;
            // what a program makes itself gets its own. (D371)
            // trace: D322, D332, D371
            NodeKind::Allocator => Ok(Value::Struct(vec![Value::Integer(0)])),
            // trace: D322, D323
            NodeKind::ListDone { list, allocator } => {
                let list = self.evaluate(list, frame)?;
                let allocator = self.allocator_number(allocator, frame)?;
                match list {
                    Value::List(_, used) => check_allocator(used, allocator, "this list", span)?,
                    Value::Map => {}
                    _ => return Err(oracle_bug(span, "`done` on something that isn't a list")),
                }
                Ok(Value::Void)
            }
            NodeKind::Flush { writer, .. } => {
                let number = self.writer_number(writer, frame)?;
                self.flush_writer(number, span)?;
                Ok(Value::Void)
            }
            NodeKind::Append {
                receiver,
                shown,
                item,
                allocator,
                max,
                count_limit,
            } => {
                let allocator = match allocator {
                    Some(allocator) => Some(self.allocator_number(allocator, frame)?),
                    None => None,
                };
                self.append(
                    receiver,
                    shown,
                    item,
                    allocator,
                    *max,
                    *count_limit,
                    frame,
                    span,
                )
            }
            NodeKind::Length(value) => {
                let Value::List(items, _) = self.evaluate(value, frame)? else {
                    return Err(oracle_bug(span, "`.len` on something else"));
                };
                Ok(Value::Integer(
                    i128::try_from(items.len()).unwrap_or(i128::MAX),
                ))
            }
            // trace: D158, D20
            NodeKind::FitFloat {
                value,
                low,
                high,
                target,
                shown,
            } => {
                let value = self.evaluate(value, frame)?;
                let Some(number) = float_value(&value) else {
                    return Err(oracle_bug(span, "a bound on something else"));
                };
                if !float_in_bound(number, *low, *high) {
                    let shown_number = match value {
                        Value::F32(_) => show_float(FloatKind::F32, number),
                        _ => show_float(FloatKind::F64, number),
                    };
                    let (what, when) = match shown {
                        Some(name) => (
                            format!("`{name}` doesn't fit `{target}`"),
                            format!("`{name}` is {shown_number}"),
                        ),
                        None => (
                            format!("this doesn't fit `{target}`"),
                            format!("it is {shown_number}"),
                        ),
                    };
                    return Err(Stop::Report(
                        Report::check_failed(
                            span,
                            what,
                            "a plain `=`, `change` or argument checks a bound when the compiler can't see it fits (D20, D158)",
                        )
                        .when(when),
                    ));
                }
                Ok(value)
            }
            // `as` is the wall between `f64.ieee` and `f64`, and it's a real conversion.
            // trace: D159
            NodeKind::FloatFromIeee { value, target } => {
                let value = self.evaluate(value, frame)?;
                let Some(number) = float_value(&value) else {
                    return Err(oracle_bug(span, "`as` on something that isn't a float"));
                };
                if number.is_finite() {
                    return Ok(value);
                }
                Err(Stop::Report(Report::check_failed(
                    span,
                    format!(
                        "this is {}, so it can't become `{target}`",
                        non_finite_name(number)
                    ),
                    "`as` out of `f64.ieee` is a real conversion, and it fails on NaN or infinity (D159)",
                )))
            }
            // trace: D161
            NodeKind::FloatToStr(number, value) => {
                let value = self.evaluate(value, frame)?;
                let (Some(held), Number::Float(kind, _)) = (float_value(&value), number) else {
                    return Err(oracle_bug(span, "`as str` on something else"));
                };
                Ok(Value::Str(match non_finite_text(held) {
                    Some(word) => word.to_string(),
                    None => show_float(*kind, held),
                }))
            }
            NodeKind::IntegerToStr(value) => {
                let Value::Integer(value) = self.evaluate(value, frame)? else {
                    return Err(oracle_bug(span, "`as str` on something else"));
                };
                Ok(Value::Str(value.to_string()))
            }
            NodeKind::Crash(value) => match self.evaluate(value, frame) {
                Err(Stop::Raised(raised)) => {
                    // trace: D96
                    let mut report = Report::check_failed(
                        span,
                        format!("{}, and nothing handled it", raised.shown),
                        "a failure nothing handles stops the program (D96)",
                    );
                    if let Some(when) = raised.when {
                        report = report.when(when);
                    }
                    if raised.span != span {
                        report = report.because(raised.span, "it was raised here");
                    }
                    Err(Stop::Report(report))
                }
                other => other,
            },
            NodeKind::Fallback { value, fallback } => match self.evaluate(value, frame) {
                Err(Stop::Raised(_)) => self.evaluate(fallback, frame),
                other => other,
            },
            NodeKind::Handle { value, arms } => match self.evaluate(value, frame) {
                Err(Stop::Raised(raised)) => {
                    for arm in arms {
                        if arm.failure == raised.name {
                            // trace: D139
                            if let Some(slot) = arm.slot {
                                let named = Binding::fresh(Value::Struct(raised.data));
                                Self::set_slot(frame, slot, named, span)?;
                            }
                            return match self.run_block(&arm.body, frame) {
                                Ok(()) => Ok(Value::Void),
                                Err(Stop::Instead(value)) => Ok(value),
                                Err(other) => Err(other),
                            };
                        }
                    }
                    Err(Stop::Raised(raised))
                }
                other => other,
            },
            NodeKind::Fit {
                value,
                low,
                high,
                target,
                shown,
            } => {
                let value = self.evaluate(value, frame)?;
                let Value::Integer(number) = value else {
                    return Err(oracle_bug(span, "a bound on something else"));
                };
                if number < *low || number > *high {
                    // trace: D20
                    let what = match shown {
                        Some(name) => format!("`{name}` doesn't fit `{target}`"),
                        None => format!("this doesn't fit `{target}`"),
                    };
                    let when = match shown {
                        Some(name) => format!("`{name}` is {number}"),
                        None => format!("it is {number}"),
                    };
                    return Err(Stop::Report(
                        Report::check_failed(
                            span,
                            what,
                            "a plain `=`, `change` or argument checks a bound when the compiler can't see it fits (D20)",
                        )
                        .when(when),
                    ));
                }
                Ok(Value::Integer(number))
            }
        }
    }

    /// Which allocator an allocator's value is.
    fn allocator_number(&mut self, allocator: &Node, frame: &mut Frame) -> Result<usize, Stop> {
        match self.evaluate(allocator, frame)? {
            Value::Struct(fields) => match fields.first() {
                Some(Value::Integer(number)) => usize::try_from(*number)
                    .map_err(|_| oracle_bug(allocator.span, "an allocator with a bad number")),
                _ => Err(oracle_bug(allocator.span, "an allocator with no number")),
            },
            _ => Err(oracle_bug(allocator.span, "an allocator that isn't one")),
        }
    }

    /// Which writer a writer's value is.
    fn writer_number(&mut self, writer: &Node, frame: &mut Frame) -> Result<usize, Stop> {
        match self.evaluate(writer, frame)? {
            Value::Struct(fields) => match fields.first() {
                Some(Value::Integer(number)) => usize::try_from(*number)
                    .ok()
                    .filter(|number| *number < self.writers.len())
                    .ok_or_else(|| oracle_bug(writer.span, "a writer that was never made")),
                _ => Err(oracle_bug(writer.span, "a writer with no number")),
            },
            _ => Err(oracle_bug(writer.span, "a writer that isn't one")),
        }
    }

    /// Writes straight to the writer's stream. stdout goes first, so what was printed before
    /// shows before stderr's. (D273)
    fn write_out(&mut self, stderr: bool, bytes: &[u8], span: Span) -> Result<(), Stop> {
        let written = if stderr {
            let mut error = std::io::stderr();
            self.out.flush().is_ok() && error.write_all(bytes).is_ok() && error.flush().is_ok()
        } else {
            self.out.write_all(bytes).is_ok() && self.out.flush().is_ok()
        };
        if written {
            Ok(())
        } else {
            Err(print_failed(span))
        }
    }

    /// Every buffered writer's held output, pushed out in the order the writers were made, as
    /// a crash starts. Whether every one of them was written. (D274)
    fn flush_every_writer(&mut self, span: Span) -> bool {
        let mut all = true;
        for number in 0..self.writers.len() {
            all &= self.flush_writer(number, span).is_ok();
        }
        all
    }

    /// Pushes what a buffered writer holds out now. (D273)
    fn flush_writer(&mut self, number: usize, span: Span) -> Result<(), Stop> {
        let Some(writer) = self.writers.get_mut(number) else {
            return Err(oracle_bug(span, "a writer that was never made"));
        };
        let held = std::mem::take(&mut writer.held);
        let stderr = writer.stderr;
        if held.is_empty() {
            return Ok(());
        }
        self.write_out(stderr, &held, span)
    }

    /// A buffered writer holds output until it's full, then writes what it holds, and anything
    /// bigger than its size goes straight out. An unbuffered one writes each print straight
    /// away. (D273)
    /// trace: D273
    fn write_to(&mut self, number: usize, line: &[u8], span: Span) -> Result<(), Stop> {
        let Some(writer) = self.writers.get(number) else {
            return Err(oracle_bug(span, "a writer that was never made"));
        };
        let (stderr, size, held) = (writer.stderr, writer.size, writer.held.len());
        let Some(size) = size else {
            return self.write_out(stderr, line, span);
        };
        if held + line.len() > size {
            self.flush_writer(number, span)?;
        }
        if line.len() > size {
            return self.write_out(stderr, line, span);
        }
        if let Some(writer) = self.writers.get_mut(number) {
            writer.held.extend_from_slice(line);
        }
        Ok(())
    }

    /// Shapes each value into its hole, then writes the line to its writer. (D194, D272)
    /// trace: D95, D194, D272
    fn format_print(
        &mut self,
        writer: &Node,
        pieces: &[crate::ir::PrintPiece],
        values: &[Node],
        stored: bool,
        frame: &mut Frame,
        span: Span,
    ) -> Result<Value, Stop> {
        let number = self.writer_number(writer, frame)?;
        let mut evaluated = Vec::with_capacity(values.len());
        for value in values {
            evaluated.push(self.evaluate(value, frame)?);
        }
        let mut line = String::new();
        for piece in pieces {
            match piece {
                crate::ir::PrintPiece::Text(text) => line.push_str(text),
                crate::ir::PrintPiece::Hole {
                    value,
                    shapes,
                    span: hole_span,
                } => {
                    let Some(value) = evaluated.get(*value) else {
                        return Err(oracle_bug(span, "a hole with no value"));
                    };
                    let text = crate::print::render(value, shapes, stored, *hole_span)
                        .map_err(Stop::Report)?;
                    line.push_str(&text);
                }
            }
        }
        self.write_to(number, line.as_bytes(), span)?;
        Ok(Value::Void)
    }

    /// trace: D96
    #[allow(clippy::too_many_arguments)] // Appending has this many parts.
    fn append(
        &mut self,
        receiver: &Place,
        shown: &str,
        item: &Node,
        allocator: Option<usize>,
        max: Option<i128>,
        count_limit: i128,
        frame: &mut Frame,
        span: Span,
    ) -> Result<Value, Stop> {
        let item = self.evaluate(item, frame)?;
        let binding = self.binding_of(frame, receiver, span)?;
        let mut root = binding.root.borrow_mut();
        let Some(Value::List(items, alloc)) = walk_mut(&mut root, &binding.path) else {
            return Err(oracle_bug(span, "`append` on something that isn't a list"));
        };
        let count = i128::try_from(items.len()).unwrap_or(i128::MAX);
        if max.is_some_and(|max| count >= max) {
            return Err(Stop::Raised(Raised {
                name: String::new(),
                shown: format!("`{shown}` is full"),
                when: Some(format!("it holds {count} items")),
                data: Vec::new(),
                span,
            }));
        }
        // trace: D140
        if count >= count_limit {
            return Err(Stop::Report(Report::not_designed(
                span,
                format!("`{shown}` growing past what its `len.type` can count, with `inf` as its `len.max`"),
                "",
            )));
        }
        // A growable list remembers the allocator it first takes memory from. (D322)
        // trace: D322
        if let Some(allocator) = allocator {
            check_allocator(*alloc, allocator, &format!("`{shown}`"), span)?;
            *alloc = Some(allocator);
        }
        items.push(item);
        Ok(Value::Void)
    }
}

impl Runner<'_> {
    /// `simd_chunks`: whole chunks first, then the leftovers as the loop says. A writing loop
    /// puts each chunk back at the end of its round, dropping padding. (D299, D300, D303)
    /// trace: D299, D300, D303
    fn run_chunks(
        &mut self,
        chunks: &crate::ir::ChunkLoop,
        frame: &mut Frame,
        span: Span,
    ) -> Result<(), Stop> {
        let base = self.binding_of(frame, &chunks.place, span)?;
        let items = match walk(&base.root.borrow(), &base.path) {
            Some(Value::List(items, _)) => items.clone(),
            _ => {
                return Err(oracle_bug(
                    span,
                    "chunks of something that isn't a collection",
                ))
            }
        };
        let lanes = chunks.lanes;
        let whole = items.len() / lanes;
        let left = items.len() % lanes;
        if left != 0 && matches!(chunks.leftovers, crate::ir::Leftovers::Exact) {
            return Err(Stop::Report(
                Report::check_failed(
                    span,
                    format!("`{}` isn't a multiple of {lanes} items long", chunks.shown),
                    "`.exact` requires a multiple, checked while running for a list (D299)",
                )
                .when(format!("it holds {} items", items.len())),
            ));
        }
        let pad = match &chunks.leftovers {
            crate::ir::Leftovers::Pad(pad) if left != 0 => Some(self.evaluate(pad, frame)?),
            _ => None,
        };
        let rounds = whole + usize::from(pad.is_some());
        for round in 0..rounds {
            let start = round * lanes;
            let taken = lanes.min(items.len() - start);
            let mut chunk: Vec<Value> = items[start..start + taken].to_vec();
            if let Some(pad) = &pad {
                chunk.resize(lanes, pad.clone());
            }
            Self::set_slot(
                frame,
                chunks.slot,
                Binding::fresh(Value::List(chunk, None)),
                span,
            )?;
            let stop = self.run_loop_body(chunks.loop_id, &chunks.body, frame, span)?;
            if chunks.writes {
                Self::write_back(frame, chunks.slot, &base, start, taken, span)?;
            }
            if stop {
                return Ok(());
            }
        }
        if let crate::ir::Leftovers::Apart { slot, body } = &chunks.leftovers {
            for (index, item) in items.iter().enumerate().skip(whole * lanes) {
                Self::set_slot(frame, *slot, Binding::fresh(item.clone()), span)?;
                let stop = self.run_loop_body(chunks.loop_id, body, frame, span)?;
                if chunks.writes {
                    let value = Self::slot_value(frame, *slot, span)?;
                    Self::put_item(&base, index, value, span)?;
                }
                if stop {
                    break;
                }
            }
        }
        Ok(())
    }

    /// What a slot holds right now.
    fn slot_value(frame: &Frame, slot: usize, span: Span) -> Result<Value, Stop> {
        let Some(Some(binding)) = frame.slots.get(slot) else {
            return Err(oracle_bug(span, "a chunk with no slot"));
        };
        let root = binding.root.borrow();
        walk(&root, &binding.path)
            .cloned()
            .ok_or_else(|| oracle_bug(span, "a chunk that isn't there"))
    }

    /// Puts a changed chunk's lanes back into the collection, leaving out any padding.
    fn write_back(
        frame: &Frame,
        slot: usize,
        base: &Binding,
        start: usize,
        taken: usize,
        span: Span,
    ) -> Result<(), Stop> {
        let Value::List(lanes, _) = Self::slot_value(frame, slot, span)? else {
            return Err(oracle_bug(span, "a chunk that isn't lanes"));
        };
        for (offset, value) in lanes.into_iter().take(taken).enumerate() {
            Self::put_item(base, start + offset, value, span)?;
        }
        Ok(())
    }

    fn put_item(base: &Binding, index: usize, value: Value, span: Span) -> Result<(), Stop> {
        let mut root = base.root.borrow_mut();
        let Some(Value::List(items, _)) = walk_mut(&mut root, &base.path) else {
            return Err(oracle_bug(
                span,
                "writing back into something that isn't a collection",
            ));
        };
        let Some(found) = items.get_mut(index) else {
            return Err(oracle_bug(span, "writing back past the end"));
        };
        *found = value;
        Ok(())
    }

    /// Making, reading, writing and moving a pointer. A pointer runs as its number and how
    /// many bytes it has moved. (D315, D316)
    /// trace: D315, D316
    fn evaluate_pointer(&mut self, node: &Node, frame: &mut Frame) -> Result<Value, Stop> {
        let span = node.span;
        let pointer = |number: usize, moved: i128| {
            Value::Struct(vec![
                Value::Integer(i128::try_from(number).unwrap_or(i128::MAX)),
                Value::Integer(moved),
            ])
        };
        match &node.kind {
            NodeKind::PointerOf(place) => {
                let binding = self.binding_of(frame, place, span)?;
                self.pointers.push(PointerTarget::Variable(binding));
                Ok(pointer(self.pointers.len() - 1, 0))
            }
            NodeKind::PointerAt(address) => {
                self.evaluate(address, frame)?;
                self.pointers.push(PointerTarget::Address);
                Ok(pointer(self.pointers.len() - 1, 0))
            }
            NodeKind::PointerMoved {
                pointer: from,
                bytes,
            } => {
                let (number, moved) = self.pointer_parts(from, frame)?;
                let Value::Integer(bytes) = self.evaluate(bytes, frame)? else {
                    return Err(oracle_bug(
                        span,
                        "a pointer moved by something that isn't a number",
                    ));
                };
                Ok(pointer(number, moved.wrapping_add(bytes)))
            }
            NodeKind::PointerRead(from) => {
                let binding = self.pointer_target(from, frame, span)?;
                let root = binding.root.borrow();
                walk(&root, &binding.path)
                    .cloned()
                    .ok_or_else(|| oracle_bug(span, "a pointer to nothing"))
            }
            NodeKind::PointerWrite { pointer: to, value } => {
                let value = self.evaluate(value, frame)?;
                let binding = self.pointer_target(to, frame, span)?;
                let mut root = binding.root.borrow_mut();
                let Some(found) = walk_mut(&mut root, &binding.path) else {
                    return Err(oracle_bug(span, "a pointer to nothing"));
                };
                *found = value;
                Ok(Value::Void)
            }
            _ => Err(oracle_bug(span, "something that isn't pointer work")),
        }
    }

    fn pointer_parts(&mut self, pointer: &Node, frame: &mut Frame) -> Result<(usize, i128), Stop> {
        match self.evaluate(pointer, frame)? {
            Value::Struct(fields) => match fields.as_slice() {
                [Value::Integer(number), Value::Integer(moved)] => Ok((
                    usize::try_from(*number)
                        .map_err(|_| oracle_bug(pointer.span, "a pointer with a bad number"))?,
                    *moved,
                )),
                _ => Err(oracle_bug(pointer.span, "a pointer that isn't one")),
            },
            _ => Err(oracle_bug(pointer.span, "a pointer that isn't one")),
        }
    }

    /// The variable a pointer reads or writes. One from `.at`, or one moved off its variable,
    /// points at memory the oracle doesn't have. (D315, D316)
    fn pointer_target(
        &mut self,
        pointer: &Node,
        frame: &mut Frame,
        span: Span,
    ) -> Result<Binding, Stop> {
        let (number, moved) = self.pointer_parts(pointer, frame)?;
        match self.pointers.get(number) {
            Some(PointerTarget::Variable(binding)) if moved == 0 => Ok(binding.clone()),
            Some(PointerTarget::Variable(_)) => Err(Stop::Report(Report::not_built(
                span,
                "reading or writing next to a variable, through a pointer moved off it",
            ))),
            Some(PointerTarget::Address) => Err(Stop::Report(Report::not_built(
                span,
                "reading or writing memory at an address, which the oracle doesn't have",
            ))),
            None => Err(oracle_bug(span, "a pointer that was never made")),
        }
    }

    /// A `simd`'s lanes, which run as a list.
    fn lanes(&mut self, node: &Node, frame: &mut Frame) -> Result<Vec<Value>, Stop> {
        match self.evaluate(node, frame)? {
            Value::List(lanes, _) => Ok(lanes),
            _ => Err(oracle_bug(node.span, "a `simd` that isn't lanes")),
        }
    }

    /// Everything that works on `simd` lanes. (D291 to D307)
    /// trace: D291, D293, D296, D297, D301, D302, D304, D306, D307
    fn evaluate_simd(&mut self, node: &Node, frame: &mut Frame) -> Result<Value, Stop> {
        let span = node.span;
        let lanes_of = |lanes: Vec<Value>| Value::List(lanes, None);
        match &node.kind {
            NodeKind::Lanewise {
                operator,
                number,
                left,
                right,
            } => {
                let left = self.lanes(left, frame)?;
                let right = self.lanes(right, frame)?;
                let mut result = Vec::with_capacity(left.len());
                for (lane, (left, right)) in left.iter().zip(&right).enumerate() {
                    let value = arithmetic(*operator, *number, left, right, span)
                        .map_err(|report| in_lane(report, lane))?;
                    result.push(value);
                }
                Ok(lanes_of(result))
            }
            NodeKind::LanewiseCompare {
                operator,
                left,
                right,
            } => {
                let left = self.lanes(left, frame)?;
                let right = self.lanes(right, frame)?;
                let mut result = Vec::with_capacity(left.len());
                for (left, right) in left.iter().zip(&right) {
                    result.push(Value::Bool(compare(*operator, left, right, span)?));
                }
                Ok(lanes_of(result))
            }
            NodeKind::Reduce { how, number, value } => {
                let lanes = self.lanes(value, frame)?;
                reduce(*how, *number, lanes, span).map_err(Stop::Report)
            }
            NodeKind::LanePick {
                condition,
                if_true,
                if_false,
            } => {
                let condition = self.lanes(condition, frame)?;
                let if_true = self.lanes(if_true, frame)?;
                let if_false = self.lanes(if_false, frame)?;
                let picked = condition
                    .iter()
                    .zip(if_true.into_iter().zip(if_false))
                    .map(|(chosen, (yes, no))| {
                        if *chosen == Value::Bool(true) {
                            yes
                        } else {
                            no
                        }
                    })
                    .collect();
                Ok(lanes_of(picked))
            }
            NodeKind::Shuffle { value, from } => {
                let lanes = self.lanes(value, frame)?;
                let mut result = Vec::with_capacity(from.len());
                for lane in from {
                    let Some(value) = lanes.get(*lane) else {
                        return Err(oracle_bug(span, "a shuffle past the lanes"));
                    };
                    result.push(value.clone());
                }
                Ok(lanes_of(result))
            }
            NodeKind::JoinLanes { left, right } => {
                let mut left = self.lanes(left, frame)?;
                left.extend(self.lanes(right, frame)?);
                Ok(lanes_of(left))
            }
            NodeKind::Lookup {
                table,
                which,
                counts_from_zero,
            } => {
                let table = self.lanes(table, frame)?;
                let which = self.lanes(which, frame)?;
                lookup(&table, &which, *counts_from_zero, span).map(lanes_of)
            }
            _ => Err(oracle_bug(span, "something that isn't `simd` work")),
        }
    }
}

/// `.%lookup`: each lane from a position worked out while running, where one past the end is a
/// bounds failure that says which lane. (D306)
fn lookup(
    table: &[Value],
    which: &[Value],
    counts_from_zero: bool,
    span: Span,
) -> Result<Vec<Value>, Stop> {
    let mut result = Vec::with_capacity(which.len());
    for (lane, position) in which.iter().enumerate() {
        let Value::Integer(position) = position else {
            return Err(oracle_bug(span, "a lookup position that isn't a number"));
        };
        let index = if counts_from_zero {
            *position
        } else {
            position - 1
        };
        let found = usize::try_from(index)
            .ok()
            .and_then(|index| table.get(index));
        let Some(found) = found else {
            let word = if counts_from_zero {
                "offset.lanes"
            } else {
                "pos"
            };
            return Err(Stop::Report(
                Report::check_failed(
                    span,
                    format!("`{word}: {position}` is outside the table"),
                    "a lane of `.%lookup` pointing past the end is a bounds failure that says which lane (D306)",
                )
                .when(format!(
                    "in lane {}, and the table has {} lanes",
                    lane + 1,
                    table.len()
                )),
            ));
        };
        result.push(found.clone());
    }
    Ok(result)
}

/// A lane's failure says which lane, counting from 1 as `pos:` does. (D293)
fn in_lane(mut report: Box<Report>, lane: usize) -> Box<Report> {
    report.when = Some(match report.when.take() {
        Some(when) => format!("in lane {}, {when}", lane + 1),
        None => format!("in lane {}", lane + 1),
    });
    report
}

/// Combining lanes in the fixed pairwise order: neighbours in pairs, then the results in pairs,
/// with an odd lane out carried up, so `.stable` floats and integer overflow come out the same
/// on every CPU. (D296, D297)
/// trace: D296, D297
fn reduce(how: Reduction, number: Option<Number>, lanes: Vec<Value>, span: Span) -> Outcome<Value> {
    let pair = |left: &Value, right: &Value| -> Outcome<Value> {
        match how {
            Reduction::AllTrue => Ok(Value::Bool(
                *left == Value::Bool(true) && *right == Value::Bool(true),
            )),
            Reduction::AnyTrue => Ok(Value::Bool(
                *left == Value::Bool(true) || *right == Value::Bool(true),
            )),
            Reduction::Sum | Reduction::Product => {
                let operator = if how == Reduction::Sum {
                    Arithmetic::Add
                } else {
                    Arithmetic::Multiply
                };
                let number = number.ok_or_else(|| {
                    Report::check_failed(
                        span,
                        "the oracle got lost: maths on bool lanes",
                        "this is a bug in the oracle, not in your program (rule 6)",
                    )
                })?;
                arithmetic(operator, number, left, right, span)
            }
            Reduction::Min | Reduction::Max => {
                let below = compare(CompareOperator::Below, right, left, span).map_err(|_| {
                    Report::check_failed(
                        span,
                        "the oracle got lost: comparing lanes",
                        "this is a bug in the oracle, not in your program (rule 6)",
                    )
                })?;
                let take_right = if how == Reduction::Min {
                    below
                } else {
                    compare(CompareOperator::Above, right, left, span).unwrap_or(false)
                };
                Ok(if take_right {
                    right.clone()
                } else {
                    left.clone()
                })
            }
        }
    };
    let mut round = lanes;
    while round.len() > 1 {
        let mut next = Vec::with_capacity(round.len().div_ceil(2));
        for chunk in round.chunks(2) {
            match chunk {
                [left, right] => next.push(pair(left, right)?),
                [carried] => next.push(carried.clone()),
                _ => {}
            }
        }
        round = next;
    }
    round.into_iter().next().ok_or_else(|| {
        Report::check_failed(
            span,
            "the oracle got lost: a `simd` with no lanes",
            "this is a bug in the oracle, not in your program (rule 6)",
        )
    })
}

/// A list handed an allocator other than the one it took its memory from, a failed check
/// like a bounds check. (D322)
/// trace: D322
fn check_allocator(used: Option<usize>, given: usize, shown: &str, span: Span) -> Result<(), Stop> {
    match used {
        Some(used) if used != given => Err(Stop::Report(
            Report::check_failed(
                span,
                format!("{shown} took its memory from another allocator"),
                "a list remembers which allocator it uses, and handing it another is a failed check (D322)",
            )
            .when(format!("it uses allocator {used}, and this is allocator {given}")),
        )),
        _ => Ok(()),
    }
}

/// trace: D116
fn negate(number: Number, value: &Value, span: Span) -> Outcome<Value> {
    match (number, value) {
        (Number::Integer(kind), Value::Integer(value)) => {
            if kind.fits(-value) {
                Ok(Value::Integer(-value))
            } else {
                Err(
                    Report::not_designed(span, "what happens when unary `-` overflows", "12.3")
                        .when(format!("negating `{value}` in `{}`", kind.word())),
                )
            }
        }
        (_, Value::F32(value)) => Ok(Value::F32(-value)),
        (_, Value::F64(value)) => Ok(Value::F64(-value)),
        _ => Err(Report::check_failed(
            span,
            "the oracle got lost: `-` on something else",
            "this is a bug in the oracle, not in your program (rule 6)",
        )),
    }
}

/// Sticks at the type's bounds instead of overflowing: its written `min:` and `max:`, or the
/// hardware's. (D294)
fn saturate(word: WordOperator, left: i128, right: i128, low: i128, high: i128) -> i128 {
    let exact = match word {
        WordOperator::Plus => left.checked_add(right),
        WordOperator::Minus => left.checked_sub(right),
        WordOperator::Times => left.checked_mul(right),
    };
    // Only a product of two 64-bit numbers can pass what 128 bits hold, and its sign says
    // which bound it passed.
    let exact = exact.unwrap_or(if (left < 0) == (right < 0) {
        i128::MAX
    } else {
        i128::MIN
    });
    exact.clamp(low, high)
}

fn wrap(value: i128, kind: IntegerKind) -> i128 {
    let bits = kind.bits();
    let modulus = 1i128 << bits;
    let low = value.rem_euclid(modulus);
    let (_, high) = kind.range();
    if low > high {
        low - modulus
    } else {
        low
    }
}

fn compare(
    operator: CompareOperator,
    left: &Value,
    right: &Value,
    span: Span,
) -> Result<bool, Stop> {
    let ordering = match (left, right) {
        (Value::Integer(left), Value::Integer(right)) => left.partial_cmp(right),
        (Value::F32(left), Value::F32(right)) => left.partial_cmp(right),
        (Value::F64(left), Value::F64(right)) => left.partial_cmp(right),
        (Value::Bool(left), Value::Bool(right)) => left.partial_cmp(right),
        _ => {
            return Err(oracle_bug(
                span,
                "a comparison of things the checker refuses",
            ))
        }
    };
    // Only `f64.ieee` can hold NaN, and IEEE 754 says every comparison with it is false except
    // `!=`, which is what an ordering of `None` gives below.
    // trace: D167
    Ok(match operator {
        CompareOperator::Equal => ordering == Some(Equal),
        CompareOperator::NotEqual => ordering != Some(Equal),
        CompareOperator::Below => ordering == Some(Less),
        CompareOperator::Above => ordering == Some(Greater),
        CompareOperator::AtLeast => matches!(ordering, Some(Greater | Equal)),
        CompareOperator::AtMost => matches!(ordering, Some(Less | Equal)),
    })
}

fn oracle_lost(span: Span, what: &str) -> Box<Report> {
    Report::check_failed(
        span,
        format!("the oracle got lost: {what}"),
        "this is a bug in the oracle, not in your program (rule 6)",
    )
}

fn float_value(value: &Value) -> Option<f64> {
    match value {
        Value::F32(number) => Some(f64::from(*number)),
        Value::F64(number) => Some(*number),
        _ => None,
    }
}

/// Whether a float is inside its bound, where `-0.0` counts as `0.0`.
/// trace: D160
fn float_in_bound(value: f64, low: f64, high: f64) -> bool {
    low <= value && value <= high
}

use crate::print::non_finite_text;

fn arithmetic(
    operator: Arithmetic,
    number: Number,
    left: &Value,
    right: &Value,
    span: Span,
) -> Outcome<Value> {
    match number {
        Number::Integer(kind) => {
            let (Value::Integer(left), Value::Integer(right)) = (left, right) else {
                return Err(Report::check_failed(
                    span,
                    "the oracle got lost: integer maths on something else",
                    "this is a bug in the oracle, not in your program (rule 6)",
                ));
            };
            integer_arithmetic(operator, kind, *left, *right, span).map(Value::Integer)
        }
        Number::Float(FloatKind::F32, rules) => {
            let (Value::F32(left), Value::F32(right)) = (left, right) else {
                return Err(Report::check_failed(
                    span,
                    "the oracle got lost: float maths on something else",
                    "this is a bug in the oracle, not in your program (rule 6)",
                ));
            };
            let result = match operator {
                Arithmetic::Add => left + right,
                Arithmetic::Subtract => left - right,
                // trace: D164
                Arithmetic::Divide(_) => left / right,
                // A bound held as f64 came from an f32 type, so it fits back exactly.
                #[allow(clippy::cast_possible_truncation)]
                Arithmetic::SaturateFloat(word, low, high) => {
                    float_word(word, *left, *right).clamp(low.value() as f32, high.value() as f32)
                }
                _ => left * right,
            };
            check_float_made(
                FloatKind::F32,
                rules,
                f64::from(result),
                f64::from(*left),
                f64::from(*right),
                span,
            )?;
            Ok(Value::F32(result))
        }
        Number::Float(FloatKind::F64, rules) => {
            let (Value::F64(left), Value::F64(right)) = (left, right) else {
                return Err(Report::check_failed(
                    span,
                    "the oracle got lost: float maths on something else",
                    "this is a bug in the oracle, not in your program (rule 6)",
                ));
            };
            let result = match operator {
                Arithmetic::Add => left + right,
                Arithmetic::Subtract => left - right,
                // trace: D164
                Arithmetic::Divide(_) => left / right,
                Arithmetic::SaturateFloat(word, low, high) => {
                    float_word(word, *left, *right).clamp(low.value(), high.value())
                }
                _ => left * right,
            };
            check_float_made(FloatKind::F64, rules, result, *left, *right, span)?;
            Ok(Value::F64(result))
        }
    }
}

/// `plus`, `minus` or `times` on two floats, which saturating then clamps: an infinity it
/// reaches sticks at the bound like any other value past it. (D373)
fn float_word<
    F: std::ops::Add<Output = F> + std::ops::Sub<Output = F> + std::ops::Mul<Output = F>,
>(
    word: WordOperator,
    left: F,
    right: F,
) -> F {
    match word {
        WordOperator::Plus => left + right,
        WordOperator::Minus => left - right,
        WordOperator::Times => left * right,
    }
}

/// A float can't be NaN or infinity, so making one is a failure, and only `f64.ieee` opts out.
/// trace: D158
fn check_float_made(
    kind: FloatKind,
    rules: FloatRules,
    result: f64,
    left: f64,
    right: f64,
    span: Span,
) -> Outcome<()> {
    if result.is_finite() || rules == FloatRules::Ieee {
        return Ok(());
    }
    Err(Report::check_failed(
        span,
        format!("this makes {}", non_finite_name(result)),
        "a float can't be NaN or infinity, and making one is a failure (D158)",
    )
    .when(format!(
        "`{}` and `{}` in `{}`",
        show_float(kind, left),
        show_float(kind, right),
        kind.word()
    )))
}

fn non_finite_name(value: f64) -> &'static str {
    if value.is_nan() {
        "NaN"
    } else {
        "infinity"
    }
}

#[allow(clippy::too_many_lines)] // One arm per operator, read top to bottom.
fn integer_arithmetic(
    operator: Arithmetic,
    kind: IntegerKind,
    left: i128,
    right: i128,
    span: Span,
) -> Outcome<i128> {
    let undecided_overflow = |word: &str, section: &str| {
        Report::not_designed(
            span,
            format!("what happens when `{word}` overflows"),
            section,
        )
        .when(format!("`{left}` and `{right}` in `{}`", kind.word()))
    };
    match operator {
        Arithmetic::Add => {
            let value = left + right;
            // trace: D42
            if !kind.fits(value) {
                return Err(Report::check_failed(
                    span,
                    format!("this `+` overflows `{}`", kind.word()),
                    "a plain `+` that overflows traps in `Working` (D42)",
                )
                .when(format!("`{left} + {right}`")));
            }
            Ok(value)
        }
        Arithmetic::Subtract => {
            let value = left - right;
            if kind.fits(value) {
                Ok(value)
            } else {
                Err(undecided_overflow("-", "12.3"))
            }
        }
        Arithmetic::Multiply => match left.checked_mul(right) {
            Some(value) if kind.fits(value) => Ok(value),
            _ => Err(undecided_overflow("x", "12.3")),
        },
        // trace: D42, D295
        Arithmetic::Wrap(word) => Ok(wrap(
            match word {
                WordOperator::Plus => left.wrapping_add(right),
                WordOperator::Minus => left.wrapping_sub(right),
                // Wrapping in 128 bits keeps the low bits a narrower type wraps to.
                WordOperator::Times => left.wrapping_mul(right),
            },
            kind,
        )),
        // trace: D294
        Arithmetic::Saturate(word, low, high) => Ok(saturate(word, left, right, low, high)),
        Arithmetic::SaturateFloat(..) => Err(oracle_lost(span, "float saturating on integers")),
        Arithmetic::Divide(narrowing) => {
            division_by_zero(right, span)?;
            let value = match narrowing {
                Narrowing::Bare(_) => {
                    return Err(oracle_lost(span, "a bare `div` on integers"));
                }
                Narrowing::Trunc => left / right,
                Narrowing::Floor => {
                    left.div_euclid(right) - i128::from(right < 0 && left.rem_euclid(right) != 0)
                }
                Narrowing::Exact => {
                    if left % right != 0 {
                        return Err(Report::not_designed(
                            span,
                            "what `div.exact` does when it isn't exact",
                            "12.3",
                        )
                        .when(format!("`{left} div.exact {right}`")));
                    }
                    left / right
                }
            };
            if kind.fits(value) {
                Ok(value)
            } else {
                Err(undecided_overflow("div", ""))
            }
        }
        Arithmetic::Remainder(narrowing) => {
            division_by_zero(right, span)?;
            Ok(match narrowing {
                Narrowing::Bare(_) => {
                    return Err(oracle_lost(span, "a bare `mod`"));
                }
                Narrowing::Trunc | Narrowing::Exact => left % right,
                // trace: D97
                Narrowing::Floor => {
                    let remainder = left % right;
                    if remainder != 0 && (remainder < 0) != (right < 0) {
                        remainder + right
                    } else {
                        remainder
                    }
                }
            })
        }
        Arithmetic::BitAnd => Ok(left & right),
        Arithmetic::BitOr => Ok(left | right),
        Arithmetic::BitXor => Ok(left ^ right),
        Arithmetic::ShiftLeft | Arithmetic::ShiftRight => {
            if right < 0 || right >= i128::from(kind.bits()) {
                return Err(Report::not_designed(span, "shifting by too much", "12.3")
                    .when(format!("the shift is {right}")));
            }
            if operator == Arithmetic::ShiftRight {
                if left < 0 {
                    return Err(Report::not_designed(
                        span,
                        "whether `shr` on a negative number keeps its sign",
                        "",
                    )
                    .when(format!("`{left}`")));
                }
                return Ok(left >> right);
            }
            let value = left << right;
            if kind.fits(value) {
                Ok(value)
            } else {
                Err(undecided_overflow("shl", ""))
            }
        }
    }
}

/// trace: D26
fn division_by_zero(right: i128, span: Span) -> Outcome<()> {
    if right == 0 {
        return Err(Report::check_failed(
            span,
            "this divides by zero",
            "division by zero traps (D26)",
        ));
    }
    Ok(())
}

fn print_failed(span: Span) -> Stop {
    Stop::Raised(Raised {
        name: "print_failed".to_string(),
        shown: "`print_failed` failed".to_string(),
        when: None,
        data: Vec::new(),
        span,
    })
}

impl Runner<'_> {
    /// The tables a call passes on, with anything forwarded from this frame's own looked up,
    /// so a generic calling a generic hands on what it was told. (D235)
    fn resolve_tables(tables: &[Table], frame: &Frame, span: Span) -> Result<Vec<Table>, Stop> {
        let mut resolved = Vec::with_capacity(tables.len());
        for table in tables {
            let mut entries = Vec::with_capacity(table.len());
            for (operator, implementation) in table {
                let concrete = match implementation {
                    Implementation::Forward {
                        parameter,
                        operator: forwarded,
                    } => frame
                        .tables
                        .get(*parameter)
                        .and_then(|own| own.iter().find(|(name, _)| name == forwarded))
                        .map(|(_, found)| found.clone())
                        .ok_or_else(|| {
                            oracle_bug(span, "a generic's table missing an operation")
                        })?,
                    other => other.clone(),
                };
                entries.push((operator.clone(), concrete));
            }
            resolved.push(entries);
        }
        Ok(resolved)
    }

    /// An operation on a type parameter's values, done as the call's table says. (D235)
    /// trace: D235
    fn generic(
        &mut self,
        parameter: usize,
        operator: &str,
        left: Value,
        right: Value,
        frame: &Frame,
        span: Span,
    ) -> Result<Value, Stop> {
        let implementation = frame
            .tables
            .get(parameter)
            .and_then(|table| table.iter().find(|(name, _)| name == operator))
            .map(|(_, found)| found.clone())
            .ok_or_else(|| oracle_bug(span, "a generic's table missing an operation"))?;
        match implementation {
            Implementation::Arithmetic(arithmetic_operator, number) => {
                arithmetic(arithmetic_operator, number, &left, &right, span).map_err(Stop::Report)
            }
            Implementation::Compare(compare_operator) => {
                Ok(Value::Bool(compare(compare_operator, &left, &right, span)?))
            }
            Implementation::Function(function) => {
                let returned = self.call(
                    function,
                    vec![(0, Binding::fresh(left)), (1, Binding::fresh(right))],
                    Vec::new(),
                    span,
                )?;
                let root = returned.root.borrow();
                walk(&root, &returned.path)
                    .cloned()
                    .ok_or_else(|| oracle_bug(span, "an operator handed back nothing"))
            }
            Implementation::Forward { .. } => {
                Err(oracle_bug(span, "a generic's table still forwarding"))
            }
        }
    }
}

/// A level in decibels of a reference, or an amount from one: `reference x 10^(value / 10)`
/// when `to_amount`, and `10 x log10(value / reference)` when not. An amount at or below zero
/// has no level, and that's a failed check. (D251, D255)
/// trace: D251, D255
fn decibels(
    value: &Value,
    number: Number,
    reference: f64,
    to_amount: bool,
    span: Span,
) -> Result<Value, Stop> {
    let given = match value {
        Value::F64(value) => *value,
        Value::F32(value) => f64::from(*value),
        _ => return Err(oracle_bug(span, "decibels of something that isn't a float")),
    };
    let result = if to_amount {
        reference * 10_f64.powf(given / 10.0)
    } else {
        10.0 * (given / reference).log10()
    };
    if !result.is_finite() {
        return Err(Stop::Report(Report::check_failed(
            span,
            format!("`{given}` has no level in decibels"),
            "a level in decibels is of an amount above zero (D251)",
        )));
    }
    Ok(match number {
        #[allow(clippy::cast_possible_truncation)] // An `f32` rounds to the nearest. (D255)
        Number::Float(crate::types::FloatKind::F32, _) => Value::F32(result as f32),
        _ => Value::F64(result),
    })
}
