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

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Type {
    Integer {
        kind: IntegerKind,
        bounds: Option<Bounds>,
    },
    Float {
        bits: u32,
        kind: FloatKind,
        bounds: Option<FloatBounds>,
    },
    Bool,
    Str,
    Void,
    Writer(WriterKind),
    /// After an error, so nothing more is said about it.
    Unknown,
}

impl Type {
    pub const I32: Type = Type::Integer { kind: IntegerKind::I32, bounds: None };

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
        matches!(self, Type::Integer { .. } | Type::Float { .. } | Type::Bool)
    }

    /// The same type with no bounds, which is what arithmetic gives back (D20).
    #[must_use]
    pub fn plain(&self) -> Type {
        match self {
            Type::Integer { kind, .. } => Type::Integer { kind: *kind, bounds: None },
            Type::Float { bits, kind, .. } => Type::Float { bits: *bits, kind: *kind, bounds: None },
            other => other.clone(),
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
            Type::Str => checked::MachineType::Text,
            Type::Void | Type::Unknown => checked::MachineType::Void,
        }
    }
}

impl fmt::Display for Type {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Integer { kind, bounds } => {
                write!(formatter, "{}", kind.name())?;
                if let Some(bounds) = bounds {
                    write!(formatter, "[min: {}, max: {}]", bounds.min, bounds.max)?;
                }
                Ok(())
            }
            Type::Float { bits, kind, bounds } => {
                write!(formatter, "f{bits}.{}", kind.word())?;
                if let Some(bounds) = bounds {
                    let min = super::floats::shortest(bounds.min(), *bits);
                    let max = super::floats::shortest(bounds.max(), *bits);
                    write!(formatter, "[min: {min}, max: {max}]")?;
                }
                Ok(())
            }
            Type::Bool => write!(formatter, "bool"),
            Type::Str => write!(formatter, "str"),
            Type::Void => write!(formatter, "void"),
            Type::Writer(writer) => write!(formatter, "{}", writer.name()),
            Type::Unknown => write!(formatter, "?"),
        }
    }
}
