//! Cyborg's types and runtime values, as the checker and the runner both see them.

use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IntegerKind {
    I32,
    I64,
    U8,
    U32,
    U64,
}

impl IntegerKind {
    pub fn word(self) -> &'static str {
        match self {
            IntegerKind::I32 => "i32",
            IntegerKind::I64 => "i64",
            IntegerKind::U8 => "u8",
            IntegerKind::U32 => "u32",
            IntegerKind::U64 => "u64",
        }
    }

    pub fn range(self) -> (i128, i128) {
        match self {
            IntegerKind::I32 => (i128::from(i32::MIN), i128::from(i32::MAX)),
            IntegerKind::I64 => (i128::from(i64::MIN), i128::from(i64::MAX)),
            IntegerKind::U8 => (0, 255),
            IntegerKind::U32 => (0, i128::from(u32::MAX)),
            IntegerKind::U64 => (0, i128::from(u64::MAX)),
        }
    }

    pub fn bits(self) -> u32 {
        match self {
            IntegerKind::U8 => 8,
            IntegerKind::I32 | IntegerKind::U32 => 32,
            IntegerKind::I64 | IntegerKind::U64 => 64,
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
    Struct(String),
    Union(String),
    List {
        item: Box<Type>,
        min: Option<i128>,
        max: Option<i128>,
        /// What `$xs.len` is, counting `items`, and how the list stores its length. (D140)
        len_type: IntegerKind,
    },
    Array {
        item: Box<Type>,
        len: i128,
        len_type: IntegerKind,
    },
    Map {
        ordered: bool,
        key: Box<Type>,
        value: Box<Type>,
    },
    Borrow {
        mutable: bool,
        sources: Sources,
        inner: Box<Type>,
    },
    /// What `case negative $f:` names: a failure, whose data its fields read. (D139)
    Failure {
        name: String,
        fields: Vec<(String, Type)>,
    },
    /// `type.new`: the same data as `underlying`, and a different type. (D134)
    New {
        name: String,
        underlying: Box<Type>,
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

    /// These are always copied, and take no ownership word.
    /// trace: D2, D123
    pub fn is_always_copied(&self) -> bool {
        match self {
            Type::Integer { .. } | Type::Float { .. } | Type::Bool => true,
            Type::New { underlying, .. } => underlying.is_always_copied(),
            _ => false,
        }
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

            Type::Struct(name) | Type::Union(name) | Type::New { name, .. } => {
                write!(formatter, "{name}")
            }
            Type::List {
                item,
                min,
                max,
                len_type,
            } => {
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
                if *len_type != IntegerKind::U64 {
                    write!(formatter, ", len.type: {}", len_type.word())?;
                }
                write!(formatter, "]")
            }
            Type::Array {
                item,
                len,
                len_type,
            } => {
                write!(formatter, "std::array<{item}>[len: {len} items")?;
                if *len_type != IntegerKind::U64 {
                    write!(formatter, ", len.type: {}", len_type.word())?;
                }
                write!(formatter, "]")
            }
            Type::Map {
                ordered,
                key,
                value,
            } => {
                let narrowing = if *ordered { "ordered" } else { "hash" };
                write!(formatter, "std::map.{narrowing}<key: {key}, val: {value}>")
            }
            Type::Failure { name, .. } => write!(formatter, "the failure `{name}`"),
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
