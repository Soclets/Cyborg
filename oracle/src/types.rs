//! Cyborg's types and runtime values, as the checker and the runner both see them.

use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IntegerKind {
    I32,
    I64,
    U8,
}

impl IntegerKind {
    pub fn word(self) -> &'static str {
        match self {
            IntegerKind::I32 => "i32",
            IntegerKind::I64 => "i64",
            IntegerKind::U8 => "u8",
        }
    }

    pub fn range(self) -> (i128, i128) {
        match self {
            IntegerKind::I32 => (i128::from(i32::MIN), i128::from(i32::MAX)),
            IntegerKind::I64 => (i128::from(i64::MIN), i128::from(i64::MAX)),
            IntegerKind::U8 => (0, 255),
        }
    }

    pub fn bits(self) -> u32 {
        match self {
            IntegerKind::I32 => 32,
            IntegerKind::I64 => 64,
            IntegerKind::U8 => 8,
        }
    }

    pub fn fits(self, value: i128) -> bool {
        let (low, high) = self.range();
        low <= value && value <= high
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FloatKind {
    F32,
    F64,
}

impl FloatKind {
    pub fn word(self) -> &'static str {
        match self {
            FloatKind::F32 => "f32",
            FloatKind::F64 => "f64",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Sources {
    Program,
    Variables(Vec<String>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Type {
    Void,
    Bool,
    Str,
    Integer {
        kind: IntegerKind,
        bound: Option<(i128, i128)>,
        unit: Option<String>,
    },
    Float {
        kind: FloatKind,
        unit: Option<String>,
    },
    /// What `$xs.len` gives. It counts `items`, and its width isn't stated, so it stays apart from
    /// the integers until the design session says otherwise.
    Length,
    Struct(String),
    Union(String),
    List {
        item: Box<Type>,
        min: Option<i128>,
        max: Option<i128>,
    },
    Array {
        item: Box<Type>,
        len: i128,
    },
    Map {
        key: Box<Type>,
        value: Box<Type>,
    },
    Borrow {
        mutable: bool,
        sources: Sources,
        inner: Box<Type>,
    },
}

impl Type {
    pub fn plain_integer(kind: IntegerKind) -> Type {
        Type::Integer {
            kind,
            bound: None,
            unit: None,
        }
    }

    pub const I32: Type = Type::Integer {
        kind: IntegerKind::I32,
        bound: None,
        unit: None,
    };

    /// These are always copied, and take no ownership word.
    /// trace: D2, D123
    pub fn is_always_copied(&self) -> bool {
        matches!(
            self,
            Type::Integer { .. } | Type::Float { .. } | Type::Bool | Type::Length
        )
    }
}

fn write_items_bound(formatter: &mut fmt::Formatter<'_>, value: i128) -> fmt::Result {
    write!(formatter, "{value} items")
}

impl fmt::Display for Type {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Void => write!(formatter, "void"),
            Type::Bool => write!(formatter, "bool"),
            Type::Str => write!(formatter, "str"),
            Type::Integer { kind, bound, unit } => {
                write!(formatter, "{}", kind.word())?;
                if let Some(unit) = unit {
                    write!(formatter, ".{unit}")?;
                }
                if let Some((low, high)) = bound {
                    write!(formatter, "[min: {low}, max: {high}]")?;
                }
                Ok(())
            }
            Type::Float { kind, unit } => {
                write!(formatter, "{}", kind.word())?;
                if let Some(unit) = unit {
                    write!(formatter, ".{unit}")?;
                }
                Ok(())
            }
            Type::Length => write!(formatter, "a length in items"),
            Type::Struct(name) | Type::Union(name) => write!(formatter, "{name}"),
            Type::List { item, min, max } => {
                write!(formatter, "std::list<{item}>[")?;
                if let Some(min) = min {
                    write!(formatter, "len.min: ")?;
                    write_items_bound(formatter, *min)?;
                    write!(formatter, ", ")?;
                }
                write!(formatter, "len.max: ")?;
                match max {
                    Some(max) => write_items_bound(formatter, *max)?,
                    None => write!(formatter, "inf items")?,
                }
                write!(formatter, "]")
            }
            Type::Array { item, len } => {
                write!(formatter, "std::array<{item}>[len: {len} items]")
            }
            Type::Map { key, value } => {
                write!(formatter, "std::map<key: {key}, val: {value}>")
            }
            Type::Borrow {
                mutable,
                sources,
                inner,
            } => {
                write!(
                    formatter,
                    "{} from ",
                    if *mutable { "Lend.mut" } else { "Lend" }
                )?;
                match sources {
                    Sources::Program => write!(formatter, "program")?,
                    Sources::Variables(names) => {
                        if let [only] = names.as_slice() {
                            write!(formatter, "${only}")?;
                        } else {
                            let joined: Vec<String> =
                                names.iter().map(|name| format!("${name}")).collect();
                            write!(formatter, "<{}>", joined.join(", "))?;
                        }
                    }
                }
                write!(formatter, " {inner}")
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Void,
    Bool(bool),
    Integer(i128),
    F32(f32),
    F64(f64),
    Str(String),
    Struct(Vec<Value>),
    /// A union's case by its index, with its fields.
    Case(usize, Vec<Value>),
    /// Lists and arrays look the same once running.
    List(Vec<Value>),
    Map,
}

impl Value {
    /// How a value shows in `when:`. Only the simple ones are shown.
    pub fn shown(&self) -> Option<String> {
        match self {
            Value::Bool(value) => Some(value.to_string()),
            Value::Integer(value) => Some(value.to_string()),
            Value::F32(value) => Some(value.to_string()),
            Value::F64(value) => Some(value.to_string()),
            _ => None,
        }
    }
}
