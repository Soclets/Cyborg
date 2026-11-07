//! Runs checked code, with every check on, as `Working` would. (D62)

use std::cell::RefCell;
use std::cmp::Ordering::{Equal, Greater, Less};
use std::io::Write;
use std::rc::Rc;

use crate::ast::{CompareOperator, Narrowing};
use crate::ir::{
    Argument, Arithmetic, Code, Contract, Function, Instruction, InstructionKind, Node, NodeKind,
    Number, Place, Position, RangeCode, Step, Test,
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
            Value::Struct(fields) | Value::Case(_, fields) | Value::List(fields) => {
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
            Value::Struct(fields) | Value::Case(_, fields) | Value::List(fields) => {
                fields.get_mut(*step)?
            }
            _ => return None,
        };
    }
    Some(current)
}

struct Frame {
    slots: Vec<Option<Binding>>,
}

struct Runner<'c> {
    code: &'c Code,
    globals: Vec<Value>,
    depth: usize,
    steps: u64,
    step_limit: Option<u64>,
    out: std::io::BufWriter<std::io::Stdout>,
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
    let mut frame = Frame { slots: Vec::new() };
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
            Err(Stop::Return(_, span)) => {
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
    let mut frame = Frame { slots: Vec::new() };
    for node in &code.globals {
        match runner.evaluate(node, &mut frame) {
            Ok(value) => runner.globals.push(value),
            Err(Stop::Report(report)) => return Err(report),
            Err(_) => return Ok(()),
        }
    }
    let result = runner.call(code.main, Vec::new(), Span::new(0, 0));
    let flushed = runner.out.flush();
    match result {
        Ok(_) => flushed.map_err(|error| {
            Report::check_failed(
                Span::new(0, 0),
                format!("the program's output couldn't be written: {error}"),
                "a failure nothing handles stops the program (D96)",
            )
        }),
        Err(Stop::Report(report)) => Err(report),
        Err(Stop::Raised(raised)) => Err(Report::check_failed(
            raised.span,
            format!("{}, and nothing handled it", raised.shown),
            "a failure nothing handles stops the program (D96)",
        )),
        Err(Stop::Return(..) | Stop::Break(_) | Stop::Instead(_)) => Ok(()),
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
        }
    }

    fn call(
        &mut self,
        function: usize,
        arguments: Vec<(usize, Binding)>,
        call_span: Span,
    ) -> Result<Value, Stop> {
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
    ) -> Result<Value, Stop> {
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
            Err(Stop::Return(value, span)) => (value, span),
            Err(other) => return Err(other),
        };
        if !function.ensures.is_empty() {
            if let Some(slot) = frame.slots.get_mut(function.result_slot) {
                *slot = Some(Binding::fresh(returned.0.clone()));
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

    fn run_block(&mut self, body: &[Instruction], frame: &mut Frame) -> Result<(), Stop> {
        for instruction in body {
            self.run_instruction(instruction, frame)?;
        }
        Ok(())
    }

    /// Works out where a place is right now. An item's position runs here, and is checked here.
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
                            Some(Value::List(items)) => items.len(),
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
            InstructionKind::Let { slot, value } => {
                let value = self.evaluate(value, frame)?;
                Self::set_slot(frame, *slot, Binding::fresh(value), span)
            }
            InstructionKind::LetAlias { slot, place } => {
                let binding = self.binding_of(frame, place, span)?;
                Self::set_slot(frame, *slot, binding, span)
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
                let Value::List(items) = self.evaluate(collection, frame)? else {
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
                float_in_bound(number, *low, *high, span).map_err(Stop::Report)
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
                let Value::List(items) = value else {
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
                Ok(Value::List(values))
            }
            NodeKind::All(value, count) => {
                let value = self.evaluate(value, frame)?;
                Ok(Value::List(vec![value; *count]))
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
            NodeKind::Call {
                function,
                arguments,
            } => {
                let mut bindings = Vec::with_capacity(arguments.len());
                for (parameter, argument) in arguments {
                    let binding = match argument {
                        Argument::Value(value) => Binding::fresh(self.evaluate(value, frame)?),
                        Argument::Alias(place) => self.binding_of(frame, place, span)?,
                    };
                    bindings.push((*parameter, binding));
                }
                self.call(*function, bindings, span)
            }
            NodeKind::Println(pieces) => self.println(pieces, frame, span),
            NodeKind::Append {
                receiver,
                shown,
                item,
                max,
                count_limit,
            } => self.append(receiver, shown, item, *max, *count_limit, frame, span),
            NodeKind::Length(value) => {
                let Value::List(items) = self.evaluate(value, frame)? else {
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
                if !float_in_bound(number, *low, *high, span).map_err(Stop::Report)? {
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

    /// trace: D95
    fn println(&mut self, pieces: &[Node], frame: &mut Frame, span: Span) -> Result<Value, Stop> {
        let mut line = String::new();
        for piece in pieces {
            let Value::Str(text) = self.evaluate(piece, frame)? else {
                return Err(oracle_bug(span, "a printed piece that isn't a `str`"));
            };
            line.push_str(&text);
        }
        line.push('\n');
        if self.out.write_all(line.as_bytes()).is_err() || self.out.flush().is_err() {
            return Err(Stop::Raised(Raised {
                name: "print_failed".to_string(),
                shown: "`print_failed` failed".to_string(),
                when: None,
                data: Vec::new(),
                span,
            }));
        }
        Ok(Value::Void)
    }

    /// trace: D96
    #[allow(clippy::too_many_arguments)] // Appending has this many parts.
    fn append(
        &mut self,
        receiver: &Place,
        shown: &str,
        item: &Node,
        max: Option<i128>,
        count_limit: i128,
        frame: &mut Frame,
        span: Span,
    ) -> Result<Value, Stop> {
        let item = self.evaluate(item, frame)?;
        let binding = self.binding_of(frame, receiver, span)?;
        let mut root = binding.root.borrow_mut();
        let Some(Value::List(items)) = walk_mut(&mut root, &binding.path) else {
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
        items.push(item);
        Ok(Value::Void)
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
    Ok(match operator {
        CompareOperator::Equal => ordering == Some(Equal),
        CompareOperator::NotEqual => ordering != Some(Equal),
        CompareOperator::Less => ordering == Some(Less),
        CompareOperator::Greater => ordering == Some(Greater),
        CompareOperator::AtLeast => matches!(ordering, Some(Greater | Equal)),
        CompareOperator::AtMost => matches!(ordering, Some(Less | Equal)),
    })
}

fn float_value(value: &Value) -> Option<f64> {
    match value {
        Value::F32(number) => Some(f64::from(*number)),
        Value::F64(number) => Some(*number),
        _ => None,
    }
}

/// Whether a float is inside its bound. How `-0.0` compares with `0.0` isn't decided, and it's the
/// one case where the answer depends on that.
/// trace: D158
fn float_in_bound(value: f64, low: f64, high: f64, span: Span) -> Outcome<bool> {
    let other_zero = |edge: f64| {
        value == 0.0 && edge == 0.0 && value.is_sign_negative() != edge.is_sign_negative()
    };
    if other_zero(low) || other_zero(high) {
        return Err(Report::not_designed(
            span,
            "whether `-0.0` is inside a bound that stops at `0.0`",
            "12.2",
        ));
    }
    Ok(low <= value && value <= high)
}

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
                _ => left * right,
            };
            check_float_made(FloatKind::F64, rules, result, *left, *right, span)?;
            Ok(Value::F64(result))
        }
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
        // trace: D42
        Arithmetic::PlusWrap => Ok(wrap(left + right, kind)),
        Arithmetic::Divide(narrowing) => {
            division_by_zero(right, span)?;
            let value = match narrowing {
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
