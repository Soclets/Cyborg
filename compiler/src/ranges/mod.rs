//! Dropping checks that can't fail (D312, rule 10): `OKAY\n` works out the smallest and largest value of each integer
//! from literals and bounded types, and drops an overflow check, a bound check, a divisor's check or an item's position
//! check when the proven decisions in `checks` say it can never fire. It runs in every mode but `Working`, which keeps
//! every check, proven or not, so a wrong proof shows up as a crash while developing (D280).
//!
//! It follows each local's range through `let`, `change`, `if` and loops: a test narrows its branches, as `$i below
//! $n` caps `$i` below `$n`'s largest value, paths join where they meet, and a loop's head is widened until it holds
//! every round, so a `$i + 1` inside a loop that runs while `$i below $n` loses its check (rule 10). A local whose
//! address is taken, by a borrow or a pointer, is never followed, since something else may change it.

pub mod checks;

use std::collections::{HashMap, HashSet};

use checks::Interval;

use crate::checked::{
    Argument, Arithmetic, AtomicOperation, Block, Comparison, Expression, ExpressionKind, LocalId, MachineType,
    OnFailure, Piece, Place, Program, Statement, Step,
};

/// Drops every check in the program that its ranges prove can't fail.
pub fn drop_proven(program: &mut Program) {
    let structs = program.structs.clone();
    for function in &mut program.functions {
        if function.template {
            continue;
        }
        // A first pass, changing nothing, finds the locals whose addresses are taken.
        let mut finder = Walker {
            locals: &function.locals,
            bounds: &function.local_bounds,
            structs: &structs,
            held: HashMap::new(),
            taken: HashSet::new(),
            dry: true,
        };
        finder.block(&mut function.body);
        let taken = finder.taken;
        let mut walker = Walker {
            locals: &function.locals,
            bounds: &function.local_bounds,
            structs: &structs,
            held: HashMap::new(),
            taken,
            dry: false,
        };
        walker.block(&mut function.body);
    }
}

/// The locals a block gives a value to, anywhere inside it.
fn assigned(block: &Block, found: &mut HashSet<LocalId>) {
    for statement in block {
        match statement {
            Statement::Let { local, .. } | Statement::Set { local, .. } | Statement::Instead { local, .. } => {
                found.insert(*local);
            }
            Statement::Store { place, .. } => {
                found.insert(place.local);
            }
            Statement::If { then, otherwise, .. } => {
                assigned(then, found);
                assigned(otherwise, found);
            }
            Statement::Loop { body, .. } => assigned(body, found),
            _ => {}
        }
    }
}

/// The comparison that holds when this one doesn't.
fn negated(operator: Comparison) -> Comparison {
    match operator {
        Comparison::Equal => Comparison::NotEqual,
        Comparison::NotEqual => Comparison::Equal,
        Comparison::Below => Comparison::AtLeast,
        Comparison::AtLeast => Comparison::Below,
        Comparison::Above => Comparison::AtMost,
        Comparison::AtMost => Comparison::Above,
    }
}

/// The same comparison with its sides swapped: `a below b` is `b above a`.
fn flipped(operator: Comparison) -> Comparison {
    match operator {
        Comparison::Below => Comparison::Above,
        Comparison::Above => Comparison::Below,
        Comparison::AtLeast => Comparison::AtMost,
        Comparison::AtMost => Comparison::AtLeast,
        other => other,
    }
}

/// Where two paths meet, each local has a range from either.
fn join(a: &HashMap<LocalId, Interval>, b: &HashMap<LocalId, Interval>) -> HashMap<LocalId, Interval> {
    a.iter().filter_map(|(local, range)| b.get(local).map(|other| (*local, checks::join(*range, *other)))).collect()
}

struct Walker<'a> {
    locals: &'a [MachineType],
    bounds: &'a [Option<(i128, i128)>],
    structs: &'a [Vec<MachineType>],
    /// What each local followed holds here, as a range.
    held: HashMap<LocalId, Interval>,
    /// Locals whose address is taken, which something else may change, so they're never followed.
    taken: HashSet<LocalId>,
    /// A pass that only works out ranges, as around a loop, dropping nothing.
    dry: bool,
}

/// Every value a machine integer can hold.
fn machine_range(machine_type: &MachineType) -> Option<Interval> {
    match *machine_type {
        MachineType::Integer { bits: 128, signed: true } => Some(Interval { low: i128::MIN, high: i128::MAX }),
        MachineType::Integer { bits, signed } if bits <= 64 => Some(if signed {
            let half = 1i128 << (bits - 1);
            Interval { low: -half, high: half - 1 }
        } else {
            Interval { low: 0, high: (1i128 << bits) - 1 }
        }),
        MachineType::Bool => Some(Interval { low: 0, high: 1 }),
        _ => None,
    }
}

impl Walker<'_> {
    fn local(&self, local: LocalId) -> Option<Interval> {
        if let Some(held) = self.held.get(&local)
            && !self.taken.contains(&local)
        {
            return Some(*held);
        }
        if let Some(Some((low, high))) = self.bounds.get(local) {
            return Some(Interval { low: *low, high: *high });
        }
        self.locals.get(local).and_then(machine_range)
    }

    fn block(&mut self, block: &mut Block) {
        self.flow(block);
    }

    /// Walks a block, giving whether its end can be reached.
    fn flow(&mut self, block: &mut Block) -> bool {
        for statement in block {
            if !self.statement(statement) {
                return false;
            }
        }
        true
    }

    /// What a local holds now: `None` for anything but an integer with a known range.
    fn hold(&mut self, local: LocalId, range: Option<Interval>) {
        match range {
            Some(range) if !self.taken.contains(&local) => {
                self.held.insert(local, range);
            }
            _ => {
                self.held.remove(&local);
            }
        }
    }

    /// The range an expression has here, changing nothing.
    fn peek(&self, expression: &Expression) -> Option<Interval> {
        match &expression.kind {
            ExpressionKind::Integer(value) => Some(Interval { low: *value, high: *value }),
            ExpressionKind::Local(local) => self.local(*local),
            _ => machine_range(&expression.machine_type),
        }
    }

    /// Whether a condition is known true or false here.
    fn decide(&self, condition: &Expression) -> Option<bool> {
        match &condition.kind {
            ExpressionKind::Bool(value) => Some(*value),
            ExpressionKind::Not(inner) => self.decide(inner).map(|value| !value),
            ExpressionKind::And(a, b) => match (self.decide(a), self.decide(b)) {
                (Some(false), _) | (_, Some(false)) => Some(false),
                (Some(true), Some(true)) => Some(true),
                _ => None,
            },
            ExpressionKind::Or(a, b) => match (self.decide(a), self.decide(b)) {
                (Some(true), _) | (_, Some(true)) => Some(true),
                (Some(false), Some(false)) => Some(false),
                _ => None,
            },
            ExpressionKind::Compare { operator, left, right } => {
                let (a, b) = (self.peek(left)?, self.peek(right)?);
                match operator {
                    Comparison::Below if a.high < b.low => Some(true),
                    Comparison::Below if a.low >= b.high => Some(false),
                    Comparison::Above if a.low > b.high => Some(true),
                    Comparison::Above if a.high <= b.low => Some(false),
                    Comparison::AtMost if a.high <= b.low => Some(true),
                    Comparison::AtMost if a.low > b.high => Some(false),
                    Comparison::AtLeast if a.low >= b.high => Some(true),
                    Comparison::AtLeast if a.high < b.low => Some(false),
                    _ => None,
                }
            }
            _ => None,
        }
    }

    /// Narrows what's held by a condition being `truth`, giving whether that can happen at all.
    fn refine(&mut self, condition: &Expression, truth: bool) -> bool {
        match &condition.kind {
            ExpressionKind::Bool(value) => *value == truth,
            ExpressionKind::Not(inner) => self.refine(inner, !truth),
            ExpressionKind::And(a, b) if truth => self.refine(a, true) && self.refine(b, true),
            ExpressionKind::Or(a, b) if !truth => self.refine(a, false) && self.refine(b, false),
            ExpressionKind::And(a, b) => match (self.decide(a), self.decide(b)) {
                (Some(true), _) => self.refine(b, false),
                (_, Some(true)) => self.refine(a, false),
                _ => true,
            },
            ExpressionKind::Or(a, b) => match (self.decide(a), self.decide(b)) {
                (Some(false), _) => self.refine(b, true),
                (_, Some(false)) => self.refine(a, true),
                _ => true,
            },
            ExpressionKind::Compare { operator, left, right } => {
                let operator = if truth { *operator } else { negated(*operator) };
                let (Some(a), Some(b)) = (self.peek(left), self.peek(right)) else { return true };
                if let ExpressionKind::Local(local) = left.kind
                    && !self.narrow(local, a, operator, b)
                {
                    return false;
                }
                if let ExpressionKind::Local(local) = right.kind
                    && !self.narrow(local, b, flipped(operator), a)
                {
                    return false;
                }
                true
            }
            _ => true,
        }
    }

    /// Narrows one local known to be `operator` some value in `other`, giving whether any of its values can be.
    fn narrow(&mut self, local: LocalId, range: Interval, operator: Comparison, other: Interval) -> bool {
        // trace: D312
        let narrowed = match operator {
            Comparison::Below => checks::below(range, other),
            Comparison::AtMost => checks::at_most(range, other),
            Comparison::Above => checks::above(range, other),
            Comparison::AtLeast => checks::at_least(range, other),
            Comparison::Equal => checks::at_least(checks::at_most(range, other), other),
            Comparison::NotEqual => range,
        };
        if narrowed.low > narrowed.high {
            return false;
        }
        self.hold(local, Some(narrowed));
        true
    }

    /// Walks one statement, giving whether what follows it can be reached.
    #[allow(clippy::too_many_lines)]
    fn statement(&mut self, statement: &mut Statement) -> bool {
        match statement {
            Statement::Let { local, value } | Statement::Set { local, value } | Statement::Instead { local, value } => {
                let range = self.expression(value);
                self.hold(*local, range);
                if matches!(statement, Statement::Instead { .. }) {
                    return false;
                }
            }
            Statement::Alias { local, target, .. } => {
                self.taken.insert(*local);
                self.taken.insert(*target);
            }
            Statement::AliasValue { local, value } => {
                self.taken.insert(*local);
                self.expression(value);
            }
            Statement::SetField { value, .. } | Statement::Evaluate(value) => {
                self.expression(value);
            }
            Statement::Check { condition, operands, .. } => {
                self.expression(condition);
                for operand in operands {
                    self.expression(operand);
                }
            }
            Statement::CheckAtSite { condition, operands, site, .. } => {
                self.expression(condition);
                for operand in operands {
                    self.expression(operand);
                }
                self.expression(site);
            }
            Statement::If { condition, then, otherwise } => {
                self.expression(condition);
                let before = self.held.clone();
                let then_reachable = self.refine(condition, true);
                if !then_reachable {
                    self.held.clone_from(&before);
                }
                let then_ends = self.flow(then) && then_reachable;
                let after_then = std::mem::replace(&mut self.held, before.clone());
                let otherwise_reachable = self.refine(condition, false);
                if !otherwise_reachable {
                    self.held.clone_from(&before);
                }
                let otherwise_ends = self.flow(otherwise) && otherwise_reachable;
                let after_otherwise = std::mem::take(&mut self.held);
                self.held = match (then_ends, otherwise_ends) {
                    (true, true) => join(&after_then, &after_otherwise),
                    (true, false) => after_then,
                    (false, true) => after_otherwise,
                    (false, false) => return false,
                };
            }
            Statement::Loop { body, .. } => self.walk_loop(body),
            Statement::Break(_) => return false,
            Statement::Return(value) => {
                if let Some(value) = value {
                    self.expression(value);
                }
                return false;
            }
            Statement::OpenWriter { capacity, .. } => {
                if let Some(capacity) = capacity {
                    self.expression(capacity);
                }
            }
            Statement::Print { pieces, .. } => {
                for piece in pieces {
                    if let Piece::Value { value, .. } = piece {
                        self.expression(value);
                    }
                }
            }
            Statement::Store { place, value } => {
                self.place(place);
                self.expression(value);
                self.hold(place.local, None);
            }
            Statement::Push { place, value, growth, .. } => {
                self.place(place);
                self.expression(value);
                if let Some(growth) = growth {
                    self.expression(&mut growth.allocator);
                }
            }
            Statement::ListDone { place, allocator, .. } => {
                self.taken.insert(place.local);
                self.place(place);
                self.expression(allocator);
            }
            Statement::ArenaDone { arena: value, .. } | Statement::TextDone(value) | Statement::PoolDone(value) => {
                self.expression(value);
            }
            Statement::TaskStart { pool, call, .. } => {
                self.expression(pool);
                self.expression(call);
            }
            // What a task gave back is only known once it's joined.
            Statement::TaskJoin { result, .. } => self.hold(*result, None),
            Statement::Assembly { reads, writes, .. } => {
                for (_, value) in reads {
                    self.expression(value);
                }
                for (_, local, _) in writes.iter() {
                    self.taken.insert(*local);
                }
            }
            Statement::Flush { .. }
            | Statement::Done { .. }
            | Statement::Flow(_)
            | Statement::LockAll { .. }
            | Statement::UnlockAll { .. } => {}
        }
        true
    }

    /// A loop: its head widened until it holds every round, then its body walked from there (D312).
    fn walk_loop(&mut self, body: &mut Block) {
        let mut changed = HashSet::new();
        assigned(body, &mut changed);
        let mut head = self.held.clone();
        let dry = self.dry;
        let mut settled = false;
        for _ in 0..4 {
            self.dry = true;
            self.held.clone_from(&head);
            let reaches_end = self.flow(body);
            let end = std::mem::take(&mut self.held);
            let mut next = head.clone();
            for local in &changed {
                let widened = match (head.get(local), end.get(local)) {
                    _ if !reaches_end => head.get(local).copied(),
                    (Some(start), Some(round)) => {
                        let whole = self.locals.get(*local).and_then(machine_range);
                        whole.map(|whole| Interval {
                            low: if round.low < start.low { whole.low } else { start.low },
                            high: if round.high > start.high { whole.high } else { start.high },
                        })
                    }
                    _ => None,
                };
                match widened {
                    Some(range) => next.insert(*local, range),
                    None => next.remove(local),
                };
            }
            if next == head {
                settled = true;
                break;
            }
            head = next;
        }
        self.dry = dry;
        if !settled {
            for local in &changed {
                head.remove(local);
            }
        }
        self.held.clone_from(&head);
        self.flow(body);
        // After the loop, what it changed is whatever a `break` left, which isn't followed.
        self.held = head;
        for local in &changed {
            self.held.remove(local);
        }
    }

    /// A place's items get their position checks dropped where the position is always inside an array's length.
    fn place(&mut self, place: &mut Place) {
        let mut current = self.locals.get(place.local).cloned();
        for step in &mut place.steps {
            current = match (step, current) {
                (Step::Field(index), Some(MachineType::Struct(id))) => {
                    self.structs.get(id).and_then(|fields| fields.get(*index)).cloned()
                }
                (Step::Item { position, from_one, check }, Some(MachineType::Array { item, len })) => {
                    let range = self.expression(position);
                    let (first, last) = if *from_one { (1, i128::from(len)) } else { (0, i128::from(len) - 1) };
                    // trace: D312
                    if !self.dry && range.is_some_and(|range| checks::inside(range, first, last)) {
                        *check = None;
                    }
                    Some(*item)
                }
                (Step::Item { position, .. }, _) => {
                    self.expression(position);
                    None
                }
                (Step::Field(_), _) => None,
            };
        }
    }

    fn arguments(&mut self, arguments: &mut [Argument]) {
        for argument in arguments {
            match argument {
                Argument::Value(value) | Argument::Temporary(value) => {
                    self.expression(value);
                }
                Argument::Place(place) => {
                    self.taken.insert(place.local);
                    self.place(place);
                }
                Argument::Address(local) | Argument::FieldAddress { local, .. } => {
                    self.taken.insert(*local);
                }
            }
        }
    }

    /// The range of an expression's value, if it's an integer, dropping the checks inside it that can't fail.
    #[allow(clippy::too_many_lines)]
    fn expression(&mut self, expression: &mut Expression) -> Option<Interval> {
        let machine = machine_range(&expression.machine_type);
        match &mut expression.kind {
            ExpressionKind::Integer(value) => Some(Interval { low: *value, high: *value }),
            ExpressionKind::Bool(value) => {
                let value = i128::from(*value);
                Some(Interval { low: value, high: value })
            }
            ExpressionKind::Local(local) => self.local(*local),
            ExpressionKind::Arithmetic { operator, left, right, failure, .. } => {
                let a = self.expression(left);
                let b = self.expression(right);
                let (Some(a), Some(b), Some(limits)) = (a, b, machine) else { return machine };
                // trace: D312
                // A wrapping or saturating sum that can't reach the type's ends is plain maths.
                let exact = match operator {
                    Arithmetic::Add | Arithmetic::WrappingAdd | Arithmetic::SaturatingAdd
                        if checks::sum_inside(a, b, limits.low, limits.high) =>
                    {
                        Some(Interval { low: a.low + b.low, high: a.high + b.high })
                    }
                    Arithmetic::Subtract | Arithmetic::WrappingSubtract | Arithmetic::SaturatingSubtract
                        if checks::difference_inside(a, b, limits.low, limits.high) =>
                    {
                        Some(Interval { low: a.low - b.high, high: a.high - b.low })
                    }
                    Arithmetic::Multiply | Arithmetic::WrappingMultiply | Arithmetic::SaturatingMultiply
                        if checks::product_inside(a, b, limits.low, limits.high) =>
                    {
                        Some(Interval { low: a.low * b.low, high: a.high * b.high })
                    }
                    _ => None,
                };
                let divides = matches!(
                    operator,
                    Arithmetic::DivideTruncate
                        | Arithmetic::DivideFloor
                        | Arithmetic::DivideExact
                        | Arithmetic::RemainderTruncate
                        | Arithmetic::RemainderFloor
                );
                if !self.dry && (exact.is_some() || (divides && checks::excludes(b, 0))) {
                    *failure = None;
                }
                exact.or(machine)
            }
            ExpressionKind::Negate { operand, failure } => {
                let range = self.expression(operand);
                if let (Some(range), Some(limits)) = (range, machine)
                    && limits.low < 0
                    && checks::excludes(range, limits.low)
                {
                    if !self.dry {
                        *failure = None;
                    }
                    return Some(Interval { low: -range.high, high: -range.low });
                }
                machine
            }
            ExpressionKind::Fit { value, min, max, .. } => {
                let range = self.expression(value);
                let (low, high) = (*min, *max);
                // trace: D312
                if range.is_some_and(|range| checks::inside(range, low, high)) {
                    if self.dry {
                        return range;
                    }
                    let ExpressionKind::Fit { value, .. } =
                        std::mem::replace(&mut expression.kind, ExpressionKind::Zero)
                    else {
                        return range;
                    };
                    *expression = *value;
                    return range;
                }
                range.map(|range| Interval { low: range.low.max(low), high: range.high.min(high) })
            }
            ExpressionKind::Clamp { value, min, max } => {
                let range = self.expression(value);
                let (low, high) = (*min, *max);
                Some(range.map_or(Interval { low, high }, |range| Interval {
                    low: range.low.clamp(low, high),
                    high: range.high.clamp(low, high),
                }))
            }
            ExpressionKind::ClampFloat { value, .. } | ExpressionKind::Maths { value, .. } => {
                self.expression(value);
                machine
            }
            ExpressionKind::Bind { local, value, body } => {
                if let Some(range) = self.expression(value) {
                    self.held.insert(*local, range);
                }
                self.expression(body)
            }
            ExpressionKind::Load(place) => {
                self.place(place);
                machine
            }
            ExpressionKind::PlaceAddress(place) => {
                self.taken.insert(place.local);
                self.place(place);
                machine
            }
            ExpressionKind::Compare { left, right, .. }
            | ExpressionKind::And(left, right)
            | ExpressionKind::Or(left, right)
            | ExpressionKind::FloatArithmetic { left, right, .. } => {
                self.expression(left);
                self.expression(right);
                machine
            }
            // A conversion keeps a range that fits where it goes.
            ExpressionKind::Convert(inner) => {
                let range = self.expression(inner);
                match (range, machine) {
                    (Some(range), Some(limits)) if checks::inside(range, limits.low, limits.high) => Some(range),
                    _ => machine,
                }
            }
            ExpressionKind::Not(inner)
            | ExpressionKind::BitNot(inner)
            | ExpressionKind::TextOf(inner)
            | ExpressionKind::ListLen(inner)
            | ExpressionKind::Tag(inner) => {
                self.expression(inner);
                machine
            }
            ExpressionKind::FromIeee { value, .. }
            | ExpressionKind::FitFloat { value, .. }
            | ExpressionKind::Fill { value, .. } => {
                self.expression(value);
                machine
            }
            ExpressionKind::Field { base, .. } | ExpressionKind::CaseField { base, .. } => {
                self.expression(base);
                machine
            }
            ExpressionKind::Call { arguments, .. } => {
                self.arguments(arguments);
                machine
            }
            ExpressionKind::Handle { call, on_failure, .. } => {
                self.expression(call);
                match on_failure.as_mut() {
                    OnFailure::Fallback(value) => {
                        self.expression(value);
                    }
                    OnFailure::Arms { arms, .. } => {
                        for (_, block) in arms {
                            self.block(block);
                        }
                    }
                    // What runs as `?` leaves doesn't come back, so what it holds is forgotten after.
                    OnFailure::PassUp { cleanup, .. } => {
                        let held = self.held.clone();
                        self.block(cleanup);
                        self.held = held;
                    }
                    OnFailure::Crash(_) | OnFailure::Ignore => {}
                }
                machine
            }
            ExpressionKind::Build(items)
            | ExpressionKind::ListOf(items)
            | ExpressionKind::BuildCase { fields: items, .. } => {
                for item in items {
                    self.expression(item);
                }
                machine
            }
            ExpressionKind::ListFrom { items, growth } => {
                for item in items {
                    self.expression(item);
                }
                self.expression(&mut growth.allocator);
                machine
            }
            ExpressionKind::TextCopy(value) => {
                self.expression(value);
                machine
            }
            ExpressionKind::FitText { value, fallback, .. } => {
                self.expression(value);
                if let Some(fallback) = fallback {
                    self.expression(fallback);
                }
                machine
            }
            ExpressionKind::Select { condition, then, otherwise } => {
                self.expression(condition);
                self.expression(then);
                self.expression(otherwise);
                machine
            }
            ExpressionKind::AddressAt(value)
            | ExpressionKind::PointerRead { pointer: value, .. }
            | ExpressionKind::UnionBits(value)
            | ExpressionKind::Bits(value) => {
                self.expression(value);
                machine
            }
            ExpressionKind::PointerWrite { pointer, value, .. }
            | ExpressionKind::PointerMoved { pointer, bytes: value } => {
                self.expression(pointer);
                self.expression(value);
                machine
            }
            ExpressionKind::Atomic { address, operation, .. } => {
                self.expression(address);
                match operation {
                    AtomicOperation::Load => {}
                    AtomicOperation::Swap(value)
                    | AtomicOperation::Store(value)
                    | AtomicOperation::Update { value, .. } => {
                        self.expression(value);
                    }
                    AtomicOperation::CompareSwap { expect, value } => {
                        self.expression(expect);
                        self.expression(value);
                    }
                }
                machine
            }
            ExpressionKind::Split { list, at, .. } => {
                self.place(list);
                self.expression(at);
                machine
            }
            ExpressionKind::ArrayView(place) => {
                self.place(place);
                machine
            }
            ExpressionKind::Heap | ExpressionKind::ArenaNew { .. } | ExpressionKind::Pool { .. } => machine,
            ExpressionKind::Site(_)
            | ExpressionKind::Float(_)
            | ExpressionKind::Text(_)
            | ExpressionKind::Zero
            | ExpressionKind::AddressOf { .. } => {
                if let ExpressionKind::AddressOf { local, .. } = &expression.kind {
                    self.taken.insert(*local);
                }
                machine
            }
        }
    }
}
