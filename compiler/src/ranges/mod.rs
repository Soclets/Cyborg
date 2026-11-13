//! Dropping checks that can't fail (D312, rule 10): `OKAY\n` works out the smallest and largest value of each integer
//! from literals and bounded types, and drops an overflow check, a bound check, a divisor's check or an item's position
//! check when the proven decisions in `checks` say it can never fire. It runs in every mode but `Working`, which keeps
//! every check, proven or not, so a wrong proof shows up as a crash while developing (D280).
//!
//! It knows only what types and literals say, not what a loop has done so far, so a check it can't prove stays.

pub mod checks;

use std::collections::HashMap;

use checks::Interval;

use crate::checked::{
    Argument, Arithmetic, Block, Expression, ExpressionKind, LocalId, MachineType, OnFailure, Piece, Place, Program,
    Statement, Step,
};

/// Drops every check in the program that its ranges prove can't fail.
pub fn drop_proven(program: &mut Program) {
    let structs = program.structs.clone();
    for function in &mut program.functions {
        if function.template {
            continue;
        }
        let mut walker = Walker {
            locals: &function.locals,
            bounds: &function.local_bounds,
            structs: &structs,
            held: HashMap::new(),
        };
        walker.block(&mut function.body);
    }
}

struct Walker<'a> {
    locals: &'a [MachineType],
    bounds: &'a [Option<(i128, i128)>],
    structs: &'a [Vec<MachineType>],
    /// Temporaries that hold one value, with its range.
    held: HashMap<LocalId, Interval>,
}

/// Every value a machine integer can hold.
fn machine_range(machine_type: &MachineType) -> Option<Interval> {
    match *machine_type {
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
        if let Some(held) = self.held.get(&local) {
            return Some(*held);
        }
        if let Some(Some((low, high))) = self.bounds.get(local) {
            return Some(Interval { low: *low, high: *high });
        }
        self.locals.get(local).and_then(machine_range)
    }

    fn block(&mut self, block: &mut Block) {
        for statement in block {
            self.statement(statement);
        }
    }

    fn statement(&mut self, statement: &mut Statement) {
        match statement {
            Statement::Let { value, .. }
            | Statement::Set { value, .. }
            | Statement::Instead { value, .. }
            | Statement::SetField { value, .. }
            | Statement::Evaluate(value)
            | Statement::AliasValue { value, .. } => {
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
                self.block(then);
                self.block(otherwise);
            }
            Statement::Loop { body, .. } => self.block(body),
            Statement::Return(value) => {
                if let Some(value) = value {
                    self.expression(value);
                }
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
            }
            Statement::Push { place, value, growth, .. } => {
                self.place(place);
                self.expression(value);
                if let Some(growth) = growth {
                    self.expression(&mut growth.allocator);
                }
            }
            Statement::ListDone { place, allocator, .. } => {
                self.place(place);
                self.expression(allocator);
            }
            Statement::ArenaDone { arena: value, .. } | Statement::TextDone(value) => {
                self.expression(value);
            }
            Statement::Assembly { reads, .. } => {
                for (_, value) in reads {
                    self.expression(value);
                }
            }
            Statement::Break(_)
            | Statement::Flush { .. }
            | Statement::Done { .. }
            | Statement::Alias { .. }
            | Statement::Flow(_) => {}
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
                    if range.is_some_and(|range| checks::inside(range, first, last)) {
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
                Argument::Place(place) => self.place(place),
                Argument::Address(_) | Argument::FieldAddress { .. } => {}
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
                let exact = match operator {
                    Arithmetic::Add if checks::sum_inside(a, b, limits.low, limits.high) => {
                        Some(Interval { low: a.low + b.low, high: a.high + b.high })
                    }
                    Arithmetic::Subtract if checks::difference_inside(a, b, limits.low, limits.high) => {
                        Some(Interval { low: a.low - b.high, high: a.high - b.low })
                    }
                    Arithmetic::Multiply if checks::product_inside(a, b, limits.low, limits.high) => {
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
                if exact.is_some() || (divides && checks::excludes(b, 0)) {
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
                    *failure = None;
                    return Some(Interval { low: -range.high, high: -range.low });
                }
                machine
            }
            ExpressionKind::Fit { value, min, max, .. } => {
                let range = self.expression(value);
                let (low, high) = (*min, *max);
                // trace: D312
                if range.is_some_and(|range| checks::inside(range, low, high)) {
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
            ExpressionKind::Load(place) | ExpressionKind::PlaceAddress(place) => {
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
            ExpressionKind::Not(inner)
            | ExpressionKind::BitNot(inner)
            | ExpressionKind::Convert(inner)
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
                    OnFailure::Crash(_) | OnFailure::Ignore | OnFailure::PassUp { .. } => {}
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
            ExpressionKind::TextCopy(value) | ExpressionKind::FitText { value, .. } => {
                self.expression(value);
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
            ExpressionKind::ArrayView(place) => {
                self.place(place);
                machine
            }
            ExpressionKind::Heap | ExpressionKind::ArenaNew { .. } => machine,
            ExpressionKind::Site(_)
            | ExpressionKind::Float(_)
            | ExpressionKind::Text(_)
            | ExpressionKind::Zero
            | ExpressionKind::AddressOf { .. } => machine,
        }
    }
}
