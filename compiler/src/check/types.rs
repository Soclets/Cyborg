//! Types as the checker sees them: with bounds and units, and printed the way errors show them.

use std::fmt;

use crate::checked;

/// An integer type's name (D10).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct IntegerKind {
    pub bits: u32,
    pub signed: bool,
}

impl IntegerKind {
    /// The most bytes this type's text can take, minus sign included, so `as str` with this bound can't fail (D328).
    #[must_use]
    pub fn most_text_bytes(self) -> u64 {
        let (lowest, highest) = self.range();
        let digits = |value: i128| value.unsigned_abs().to_string().len() as u64;
        digits(highest).max(digits(lowest) + u64::from(lowest < 0))
    }
    pub const I32: IntegerKind = IntegerKind { bits: 32, signed: true };
    pub const I64: IntegerKind = IntegerKind { bits: 64, signed: true };
    pub const U64: IntegerKind = IntegerKind { bits: 64, signed: false };

    /// The smallest and largest values it holds.
    #[must_use]
    pub fn range(self) -> (i128, i128) {
        if self.signed {
            let half = 1i128 << (self.bits - 1);
            (-half, half - 1)
        } else {
            (0, (1i128 << self.bits) - 1)
        }
    }

    #[must_use]
    pub fn name(self) -> String {
        format!("{}{}", if self.signed { 'i' } else { 'u' }, self.bits)
    }

    #[must_use]
    pub fn from_name(name: &str) -> Option<IntegerKind> {
        let (signed, bits) = match name.split_at_checked(1)? {
            ("i", bits) => (true, bits),
            ("u", bits) => (false, bits),
            _ => return None,
        };
        let bits: u32 = bits.parse().ok()?;
        matches!(bits, 8 | 16 | 32 | 64).then_some(IntegerKind { bits, signed })
    }
}

/// A float's kind, said straight after its type (D158).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FloatKind {
    Stable,
    Fast,
    Ieee,
}

impl FloatKind {
    #[must_use]
    pub fn word(self) -> &'static str {
        match self {
            FloatKind::Stable => "stable",
            FloatKind::Fast => "fast",
            FloatKind::Ieee => "ieee",
        }
    }
}

/// Where an `os` writer writes, and whether it's buffered (D273).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct WriterKind {
    pub stream: checked::Stream,
    pub buffered: bool,
}

impl WriterKind {
    #[must_use]
    pub fn name(self) -> String {
        let stream = match self.stream {
            checked::Stream::Out => "stdout",
            checked::Stream::Error => "stderr",
        };
        let kind = if self.buffered { "buffered" } else { "unbuffered" };
        format!("os::{stream}.{kind}")
    }
}

/// Integer bounds (D20), as numbers.
/// How a growable list grows when it's full (D385): doubling, from room for at least 4, unless its type says.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Grow {
    Double,
    Exact,
    ByItems(u64),
    ByBytes(u64),
}

/// A list of its own, or a view of some or all of another list or array, which only reads, or changes its items with
/// `.mut` (D379, D381).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ListView {
    Owned,
    Read,
    Change,
}

/// A runner: one thread switching between tasks, the OS's threads, or an event runner (D345, D359, D377).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RunnerKind {
    OneThread,
    Threads,
    Events,
}

/// The allocators a program can name (D332, D333).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AllocatorKind {
    Heap,
    Arena,
}

/// How a `str` holds its text: a view of someone else's, up to a number of bytes inside the value, or growing on the
/// heap (D326, D327).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum StrSize {
    View,
    Bytes(u64),
    Growing,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Bounds {
    pub min: i128,
    pub max: i128,
}

/// Float bounds (D158), kept as the bits of `f64`s that hold them exactly, so types compare and hash.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct FloatBounds {
    pub min: u64,
    pub max: u64,
}

impl FloatBounds {
    #[must_use]
    pub fn new(min: f64, max: f64) -> FloatBounds {
        FloatBounds { min: min.to_bits(), max: max.to_bits() }
    }

    #[must_use]
    pub fn min(self) -> f64 {
        f64::from_bits(self.min)
    }

    #[must_use]
    pub fn max(self) -> f64 {
        f64::from_bits(self.max)
    }
}

/// A number's unit (D99, D100, D242): the units it's made of, each with its power, in the order maths met them.
/// `time.seconds` is one unit to the power 1, and `length.meters per time.seconds` two.
#[derive(Clone, Debug, Eq)]
pub struct Unit {
    pub factors: Vec<(String, i32)>,
}

impl Unit {
    #[must_use]
    pub fn named(name: &str) -> Unit {
        Unit { factors: vec![(name.to_string(), 1)] }
    }

    /// The unit of a product, or with `sign` -1 a quotient. `None` when every power cancels, as in `$t div $t`.
    #[must_use]
    pub fn combine(left: Option<&Unit>, right: Option<&Unit>, sign: i32) -> Option<Unit> {
        let mut factors: Vec<(String, i32)> = left.map(|unit| unit.factors.clone()).unwrap_or_default();
        for (name, power) in right.map(|unit| unit.factors.clone()).unwrap_or_default() {
            match factors.iter_mut().find(|(each, _)| *each == name) {
                Some((_, mine)) => *mine += sign * power,
                None => factors.push((name, sign * power)),
            }
        }
        factors.retain(|(_, power)| *power != 0);
        if factors.is_empty() { None } else { Some(Unit { factors }) }
    }

    /// Whether it's one declared unit, rather than one made by maths.
    #[must_use]
    pub fn single(&self) -> Option<&str> {
        match self.factors.as_slice() {
            [(name, 1)] => Some(name),
            _ => None,
        }
    }

    fn sorted(&self) -> Vec<(String, i32)> {
        let mut factors = self.factors.clone();
        factors.sort();
        factors
    }
}

impl PartialEq for Unit {
    fn eq(&self, other: &Unit) -> bool {
        self.sorted() == other.sorted()
    }
}

// Hashed the way it compares, so equal units hash the same whatever order maths met them in.
impl std::hash::Hash for Unit {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.sorted().hash(state);
    }
}

impl fmt::Display for Unit {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let repeat =
            |power: i32, name: &str| vec![name.to_string(); usize::try_from(power.unsigned_abs()).unwrap_or(0)];
        let above: Vec<String> = self
            .factors
            .iter()
            .filter(|(_, power)| *power > 0)
            .flat_map(|(name, power)| repeat(*power, name))
            .collect();
        let below: Vec<String> = self
            .factors
            .iter()
            .filter(|(_, power)| *power < 0)
            .flat_map(|(name, power)| repeat(*power, name))
            .collect();
        let top = if above.is_empty() { "1".to_string() } else { above.join(" x ") };
        if below.is_empty() { write!(formatter, "{top}") } else { write!(formatter, "{top} per {}", below.join(" x ")) }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Type {
    Integer {
        kind: IntegerKind,
        bounds: Option<Bounds>,
        /// Its unit, if the type asks for one (D101).
        unit: Option<Unit>,
    },
    Float {
        bits: u32,
        kind: FloatKind,
        bounds: Option<FloatBounds>,
        unit: Option<Unit>,
    },
    Bool,
    /// Text: a read-only view, or owned, bounded or growing (D326, D327).
    Str(StrSize),
    Void,
    Writer(WriterKind),
    /// A struct of the program's own, by its number and name (D57).
    Struct {
        id: usize,
        name: String,
    },
    /// A tagged union of the program's own, by its number and name (D33).
    Union {
        id: usize,
        name: String,
    },
    /// A failure named in an `otherwise:` arm, which is for reading its data (D139): the union of the call's
    /// failures, the failure's name, and its fields' names and types.
    Failure {
        union: usize,
        name: String,
        fields: Vec<(String, Type)>,
    },
    /// A `type.new`: the same data as its inner type, but a type of its own (D134).
    New {
        name: String,
        inner: Box<Type>,
    },
    /// A stored borrow, `Lend from $r reading`: it reads or changes its sources in place (D2, D169).
    Borrow {
        mutable: bool,
        /// `.held`: it lasts until the end of its block, not its last use.
        held: bool,
        /// `$r`, or `program`.
        sources: Vec<String>,
        inner: Box<Type>,
    },
    /// `std::coll::array<i32>[len: 3 items]`: exactly `len` items (D12, D24).
    Array {
        item: Box<Type>,
        len: u64,
        /// What `.len` is, `u64` unless `len.type` says otherwise (D140).
        len_type: IntegerKind,
    },
    /// `std::coll::list<i32>[len.max: inf items]`: from `min` to `max` items, with no `max` for `inf` (D11, D24).
    List {
        item: Box<Type>,
        min: u64,
        max: Option<u64>,
        len_type: IntegerKind,
        /// How a growable list grows when it's full (D385).
        grow: Grow,
        /// Whether it's a list of its own or a view of someone else's (D379).
        view: ListView,
    },
    /// An allocator: the OS heap, or an arena (D321, D330, D332, D333).
    Allocator(AllocatorKind),
    /// A task from `@concur::spawn`, giving back its call's value at `@concur::task.done` (D343, D357).
    Task(Box<Type>),
    /// Where tasks run (D345, D350, D377).
    Runner(RunnerKind),
    /// `@concur::lock<T>`: data reached only by locking it (D348).
    Lock(Box<Type>),
    /// `@concur::atomic<T>`: an integer or `bool` changed by indivisible instructions (D354).
    Atomic(Box<Type>),
    /// An integer or `bool` that mustn't affect how long the program takes (D339).
    Secret(Box<Type>),
    /// A portable vector: lanes of one type, worked on all at once (D291).
    Simd {
        item: Box<Type>,
        lanes: u64,
    },
    /// A raw or volatile pointer, which lives only in a shield (D314, D317).
    Pointer {
        target: Box<Type>,
        mutable: bool,
        volatile: bool,
    },
    /// A type parameter inside its generic's own body, which can only be used as its requirement lists (D235).
    Param {
        name: String,
        /// The requirement, as written, for errors.
        needs: String,
    },
    /// After an error, so nothing more is said about it.
    Unknown,
}

impl Type {
    pub const I32: Type = Type::Integer { kind: IntegerKind::I32, bounds: None, unit: None };

    #[must_use]
    pub fn is_unknown(&self) -> bool {
        matches!(self, Type::Unknown)
    }

    #[must_use]
    pub fn is_integer(&self) -> bool {
        matches!(self, Type::Integer { .. })
    }

    /// Always copied, taking no ownership word (D184): numbers and `bool`.
    #[must_use]
    pub fn is_copied(&self) -> bool {
        match self {
            Type::New { inner, .. } | Type::Secret(inner) => inner.is_copied(),
            // A pointer is an address, copied like a number, though it still borrows what it points at (D315).
            _ => matches!(self, Type::Integer { .. } | Type::Float { .. } | Type::Bool | Type::Pointer { .. }),
        }
    }

    /// The type with any secrecy taken off, and whether it had it (D339).
    #[must_use]
    pub fn public(&self) -> (&Type, bool) {
        match self {
            Type::Secret(inner) => (inner, true),
            other => (other, false),
        }
    }

    /// The same list type, growing another way.
    #[must_use]
    pub fn with_grow(&self, grow: Grow) -> Type {
        match self {
            Type::List { item, min, max, len_type, view, .. } => {
                Type::List { item: item.clone(), min: *min, max: *max, len_type: *len_type, grow, view: *view }
            }
            other => other.clone(),
        }
    }

    /// The same type with no bounds, which is what arithmetic gives back (D20).
    #[must_use]
    pub fn plain(&self) -> Type {
        match self {
            Type::Integer { kind, unit, .. } => Type::Integer { kind: *kind, bounds: None, unit: unit.clone() },
            Type::Float { bits, kind, unit, .. } => {
                Type::Float { bits: *bits, kind: *kind, bounds: None, unit: unit.clone() }
            }
            other => other.clone(),
        }
    }

    /// A number's unit, if it has one.
    #[must_use]
    pub fn unit(&self) -> Option<&Unit> {
        match self {
            Type::Integer { unit, .. } | Type::Float { unit, .. } => unit.as_ref(),
            _ => None,
        }
    }

    /// The same number type with another unit, or none.
    #[must_use]
    pub fn with_unit(&self, new: Option<Unit>) -> Type {
        match self {
            Type::Integer { kind, bounds, .. } => Type::Integer { kind: *kind, bounds: *bounds, unit: new },
            Type::Float { bits, kind, bounds, .. } => {
                Type::Float { bits: *bits, kind: *kind, bounds: *bounds, unit: new }
            }
            other => other.clone(),
        }
    }

    /// What a value reads as: a stored borrow reads as what it borrows.
    #[must_use]
    pub fn read_as(&self) -> &Type {
        match self {
            Type::Borrow { inner, .. } => inner,
            other => other,
        }
    }

    /// The machine type.
    #[must_use]
    pub fn machine(&self) -> checked::MachineType {
        match self {
            Type::Integer { kind, .. } => checked::MachineType::Integer { bits: kind.bits, signed: kind.signed },
            Type::Float { bits, .. } => checked::MachineType::Float { bits: *bits },
            Type::Bool => checked::MachineType::Bool,
            Type::Writer(_) => checked::MachineType::Writer,
            Type::Struct { id, .. } => checked::MachineType::Struct(*id),
            Type::Union { id, .. } => checked::MachineType::Union(*id),
            Type::Failure { union, .. } => checked::MachineType::Union(*union),
            Type::New { inner, .. } | Type::Borrow { inner, .. } => inner.machine(),
            Type::Array { item, len, .. } => checked::MachineType::Array { item: Box::new(item.machine()), len: *len },
            Type::List { item, .. } => checked::MachineType::List(Box::new(item.machine())),
            Type::Str(_) => checked::MachineType::Text,
            Type::Allocator(_) | Type::Pointer { .. } => checked::MachineType::Address,
            Type::Simd { item, lanes } => checked::MachineType::Array { item: Box::new(item.machine()), len: *lanes },
            Type::Secret(inner) | Type::Lock(inner) | Type::Atomic(inner) => inner.machine(),
            Type::Task(_) | Type::Runner(_) => checked::MachineType::I64,
            Type::Void | Type::Unknown => checked::MachineType::Void,
            // A generic's own body is only checked, never built: each call builds a copy with real types.
            Type::Param { .. } => checked::MachineType::Address,
        }
    }
}

impl fmt::Display for Type {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Integer { kind, bounds, unit } => {
                write!(formatter, "{}", kind.name())?;
                let shown = |value: i128| match unit {
                    Some(unit) => format!("{value} {unit}"),
                    None => value.to_string(),
                };
                match (unit, bounds) {
                    (Some(unit), Some(bounds)) => {
                        write!(formatter, "[unit: {unit}, min: {}, max: {}]", shown(bounds.min), shown(bounds.max))
                    }
                    (Some(unit), None) => write!(formatter, "[unit: {unit}]"),
                    (None, Some(bounds)) => write!(formatter, "[min: {}, max: {}]", bounds.min, bounds.max),
                    (None, None) => Ok(()),
                }
            }
            Type::Float { bits, kind, bounds, unit } => {
                write!(formatter, "f{bits}.{}", kind.word())?;
                let with_unit = |value: String| match unit {
                    Some(unit) => format!("{value} {unit}"),
                    None => value,
                };
                match (unit, bounds) {
                    (unit, Some(bounds)) => {
                        let min = with_unit(super::floats::shortest(bounds.min(), *bits));
                        let max = with_unit(super::floats::shortest(bounds.max(), *bits));
                        match unit {
                            Some(unit) => write!(formatter, "[unit: {unit}, min: {min}, max: {max}]"),
                            None => write!(formatter, "[min: {min}, max: {max}]"),
                        }
                    }
                    (Some(unit), None) => write!(formatter, "[unit: {unit}]"),
                    (None, None) => Ok(()),
                }
            }
            Type::Bool => write!(formatter, "bool"),
            Type::Str(StrSize::View) => write!(formatter, "str"),
            Type::Str(StrSize::Bytes(most)) => write!(formatter, "str[len.max: {most} bytes]"),
            Type::Str(StrSize::Growing) => write!(formatter, "str[len.max: inf bytes]"),
            Type::Void => write!(formatter, "void"),
            Type::Writer(writer) => write!(formatter, "{}", writer.name()),
            Type::Struct { name, .. }
            | Type::Union { name, .. }
            | Type::Failure { name, .. }
            | Type::New { name, .. } => {
                write!(formatter, "{name}")
            }
            Type::Borrow { mutable, held, sources, inner } => {
                let word = match (mutable, held) {
                    (false, false) => "Lend",
                    (true, false) => "Lend.mut",
                    (false, true) => "Lend.held",
                    (true, true) => "Lend.mut.held",
                };
                match sources.as_slice() {
                    [one] => write!(formatter, "{word} from {one} {inner}"),
                    _ => write!(formatter, "{word} from <{}> {inner}", sources.join(", ")),
                }
            }
            Type::Array { item, len, len_type } => {
                write!(formatter, "std::coll::array<{item}>[len: {len} items")?;
                if *len_type != IntegerKind::U64 {
                    write!(formatter, ", len.type: {}", len_type.name())?;
                }
                write!(formatter, "]")
            }
            Type::Simd { item, lanes } => write!(formatter, "simd<{item}>[lanes: {lanes}]"),
            Type::Task(inner) => write!(formatter, "@concur::task<{inner}>"),
            Type::Lock(inner) => write!(formatter, "@concur::lock<{inner}>"),
            Type::Atomic(inner) => write!(formatter, "@concur::atomic<{inner}>"),
            Type::Runner(RunnerKind::OneThread) => write!(formatter, "std::run::one_thread"),
            Type::Runner(RunnerKind::Threads) => write!(formatter, "os::threads"),
            Type::Runner(RunnerKind::Events) => write!(formatter, "os::events"),
            Type::Secret(inner) => {
                let shown = inner.to_string();
                match shown.split_once('[') {
                    Some((base, rest)) => write!(formatter, "{base}.secret[{rest}"),
                    None => write!(formatter, "{shown}.secret"),
                }
            }
            Type::Pointer { target, mutable, volatile } => {
                let kind = if *volatile { "volatile" } else { "raw" };
                let changes = if *mutable { ".mut" } else { "" };
                write!(formatter, "ptr.{kind}{changes}<{target}>")
            }
            Type::Allocator(AllocatorKind::Heap) => write!(formatter, "os::heap"),
            Type::Allocator(AllocatorKind::Arena) => write!(formatter, "std::mem::arena"),
            Type::List { item, view: ListView::Read, .. } => write!(formatter, "std::coll::list<{item}>"),
            Type::List { item, view: ListView::Change, .. } => write!(formatter, "std::coll::list.mut<{item}>"),
            Type::List { item, min, max, len_type, grow, .. } => {
                write!(formatter, "std::coll::list<{item}>[")?;
                if *min > 0 {
                    write!(formatter, "len.min: {min} items, ")?;
                }
                match max {
                    Some(max) => write!(formatter, "len.max: {max} items")?,
                    None => write!(formatter, "len.max: inf items")?,
                }
                if *len_type != IntegerKind::U64 {
                    write!(formatter, ", len.type: {}", len_type.name())?;
                }
                match grow {
                    Grow::Double => {}
                    Grow::Exact => write!(formatter, ", grow: exact")?,
                    Grow::ByItems(step) => write!(formatter, ", grow: by {step} items")?,
                    Grow::ByBytes(step) => write!(formatter, ", grow: by {step} bytes")?,
                }
                write!(formatter, "]")
            }
            Type::Param { name, .. } => write!(formatter, "?{name}"),
            Type::Unknown => write!(formatter, "?"),
        }
    }
}
