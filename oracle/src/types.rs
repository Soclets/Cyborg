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

/// What a float allows. `ieee` and `fast` are placeholder names. (D158)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FloatRules {
    /// Never NaN or infinity, and the same bits on every machine.
    Plain,
    /// Allows NaN and infinity.
    Ieee,
    /// Allows merged and reordered maths, which a compiler may do and the oracle never does. It
    /// runs in the order written, and a compiler only has to come close.
    /// trace: D163
    Fast,
}

impl FloatRules {
    /// The narrowing that asks for these rules. This is the one place that spells them.
    /// The kind a float says straight after its type. (D257)
    pub fn from_narrowing(word: &str) -> Option<FloatRules> {
        match word {
            "stable" => Some(FloatRules::Plain),
            "ieee" => Some(FloatRules::Ieee),
            "fast" => Some(FloatRules::Fast),
            _ => None,
        }
    }

    pub fn word(self) -> &'static str {
        match self {
            FloatRules::Plain => "stable",
            FloatRules::Ieee => "ieee",
            FloatRules::Fast => "fast",
        }
    }
}

/// A float bound, held as bits so a type can still be compared with `==`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FloatBound(u64);

impl FloatBound {
    /// `-0.0` and `0.0` are the same place to stop, so they make the same type. (D160)
    pub fn new(value: f64) -> FloatBound {
        FloatBound(if value == 0.0 { 0.0_f64 } else { value }.to_bits())
    }

    pub fn value(self) -> f64 {
        f64::from_bits(self.0)
    }
}

/// A float's text, the shortest literal that means its value. It's written the way the language
/// writes floats, so a whole number keeps its `.0` and there's no exponent.
/// trace: D161
#[allow(clippy::cast_possible_truncation)] // An `f32` was widened to get here, so this is exact.
pub fn show_float(kind: FloatKind, value: f64) -> String {
    let text = match kind {
        FloatKind::F32 => format!("{}", value as f32),
        FloatKind::F64 => format!("{value}"),
    };
    if text.contains('.') {
        text
    } else {
        format!("{text}.0")
    }
}

/// Every digit a finite float actually holds, with no limit.
/// trace: D161, D162
pub fn stored_float(value: f64) -> String {
    // Writing out this many places shows a binary float's value in full.
    let full = format!("{value:.1100}");
    let trimmed = full.trim_end_matches('0');
    if trimmed.ends_with('.') {
        format!("{trimmed}0")
    } else {
        trimmed.to_string()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Sources {
    Program,
    Variables(Vec<String>),
    /// A struct's, union's or function's own source, as in ``Lend from `text` reading``. (D171)
    Named(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Type {
    Void,
    /// `?item`, a type parameter: a value a generic can only do with what its requirement
    /// lists. (D234, D235)
    Parameter(String),
    Bool,
    /// A bare `str`: a read-only view of text that owns nothing. (D327)
    Str,
    /// `str[len.max: 32 bytes]`: text held inside the value, up to that many bytes. (D326)
    BoundedStr(i128),
    /// `str[len.max: inf bytes]`: text that grows on the heap, and is `must_Give`. (D326)
    GrowableStr,
    /// `ptr.raw.mut<u32>` and the rest: a pointer, which only lives in a shield. (D314, D317)
    Pointer {
        volatile: bool,
        mutable: bool,
        target: Box<Type>,
    },
    /// `simd<i32>[lanes: 4]`: several values, its lanes, worked on together. (D291)
    Simd {
        item: Box<Type>,
        lanes: usize,
    },
    /// `u32.secret` or `bool.secret`: a value that mustn't affect how long the program takes.
    /// (D339)
    Secret(Box<Type>),
    Integer {
        kind: IntegerKind,
        bound: Option<(i128, i128)>,
        unit: Option<String>,
    },
    Float {
        kind: FloatKind,
        bound: Option<(FloatBound, FloatBound)>,
        unit: Option<String>,
        rules: FloatRules,
    },
    Struct(String),
    Union(String),
    /// `union.@csup::untagged number`, C's union, whose fields share the same bytes. (D398)
    Untagged(String),
    /// `@concur::task<i64>`: a task that gives back an `i64` when it's done. (D344)
    Task(Box<Type>),
    /// `@concur::atomic<u64>`: an integer or `bool` changed by indivisible instructions. (D354)
    Atomic(Box<Type>),
    /// `std::coll::halves.mut<i32>`: a list split in two, `.first` and `.second`, each a view
    /// of its part. (D382)
    Halves {
        item: Box<Type>,
        mutable: bool,
    },
    /// `@concur::lock<i64>`: a lock holding the data it protects. (D348)
    Lock(Box<Type>),
    /// A view of some or all of a list, which reads it, or with `.mut` changes its items but
    /// can't grow or shrink it. It owns no memory. (D379)
    ListView {
        item: Box<Type>,
        mutable: bool,
    },
    List {
        item: Box<Type>,
        min: Option<i128>,
        max: Option<i128>,
        /// What `$xs.len` is, counting `items`, and how the list stores its length. (D140)
        len_type: IntegerKind,
        /// How it grows when it's full. (D385)
        grow: Growth,
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
    /// A struct or union holding borrows, with its sources filled in, as in
    /// ``index<`text`: $article>``. Each fill is `$name`, `program` or `` `t` ``. (D171, D176)
    Holding {
        base: Box<Type>,
        sources: Vec<String>,
        fills: Vec<String>,
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

    /// Why a type is always copied, for the refusal of a word on one.
    /// trace: D368
    pub fn why_copied(&self) -> &'static str {
        match self {
            Type::Str => "a bare `str` is a view, a pointer and a length to read-only text, so copying one can't change or free anything, and it takes no word (D368)",
            Type::ListView { .. } => "a read-only list view is copied like a `str` view, since copying read-only data can't change or free anything, and it takes no word (D368, D381)",
            _ => "plain numbers and `bool` are always copied, and take no word (D2, D123)",
        }
    }

    /// These are always copied, and take no ownership word.
    /// trace: D2, D123
    pub fn is_always_copied(&self) -> bool {
        match self {
            // A pointer is an address, copied as one. (D314)
            // A bare `str` is a pointer and a length to read-only text, so copying one can't
            // change or free anything. (D368)
            // trace: D368
            Type::Integer { .. }
            | Type::Float { .. }
            | Type::Bool
            | Type::Pointer { .. }
            | Type::Str
            // A read-only list view copies freely too, and a `.mut` one can't. (D381)
            // trace: D381
            | Type::ListView { mutable: false, .. } => true,
            Type::Secret(inner) => inner.is_always_copied(),
            // A vector is a machine type like `i32`, so it's copied the same way. (D291)
            Type::Simd { item, .. } => item.is_always_copied(),
            Type::New { underlying, .. } => underlying.is_always_copied(),
            _ => false,
        }
    }
}

fn write_float(
    formatter: &mut fmt::Formatter<'_>,
    kind: FloatKind,
    bound: Option<(FloatBound, FloatBound)>,
    unit: Option<&String>,
    rules: FloatRules,
) -> fmt::Result {
    // trace: D257
    write!(formatter, "{}.{}", kind.word(), rules.word())?;
    let bound = bound.map(|(low, high)| {
        (
            show_float(kind, low.value()),
            show_float(kind, high.value()),
        )
    });
    write_brackets(formatter, unit, bound)
}

/// A number type's brackets: its unit first, then its bounds, which write the unit too.
/// (D258, D260)
/// trace: D258, D260
fn write_brackets(
    formatter: &mut fmt::Formatter<'_>,
    unit: Option<&String>,
    bound: Option<(String, String)>,
) -> fmt::Result {
    match (unit, bound) {
        (Some(unit), Some((low, high))) => write!(
            formatter,
            "[unit: {unit}, min: {low} {unit}, max: {high} {unit}]"
        ),
        (Some(unit), None) => write!(formatter, "[unit: {unit}]"),
        (None, Some((low, high))) => write!(formatter, "[min: {low}, max: {high}]"),
        (None, None) => Ok(()),
    }
}

fn write_items_bound(formatter: &mut fmt::Formatter<'_>, value: i128) -> fmt::Result {
    write!(formatter, "{value} items")
}

impl fmt::Display for Type {
    #[allow(clippy::too_many_lines)] // One arm per type, each written as a program writes it.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Void => write!(formatter, "void"),
            Type::Parameter(name) => write!(formatter, "?{name}"),
            Type::Bool => write!(formatter, "bool"),
            Type::Str => write!(formatter, "str"),
            Type::BoundedStr(bytes) => write!(formatter, "str[len.max: {bytes} bytes]"),
            Type::GrowableStr => write!(formatter, "str[len.max: inf bytes]"),
            Type::Simd { item, lanes } => write!(formatter, "simd<{item}>[lanes: {lanes}]"),
            Type::Pointer {
                volatile,
                mutable,
                target,
            } => write!(
                formatter,
                "ptr.{}{}<{target}>",
                if *volatile { "volatile" } else { "raw" },
                if *mutable { ".mut" } else { "" }
            ),
            // `.secret` goes on the type's own name, before any brackets. (D339)
            Type::Secret(inner) => match &**inner {
                Type::Integer { kind, bound, unit } => {
                    write!(formatter, "{}.secret", kind.word())?;
                    let bound = bound.map(|(low, high)| (low.to_string(), high.to_string()));
                    write_brackets(formatter, unit.as_ref(), bound)
                }
                other => write!(formatter, "{other}.secret"),
            },
            Type::Integer { kind, bound, unit } => {
                write!(formatter, "{}", kind.word())?;
                let bound = bound.map(|(low, high)| (low.to_string(), high.to_string()));
                write_brackets(formatter, unit.as_ref(), bound)
            }
            Type::Float {
                kind,
                bound,
                unit,
                rules,
            } => write_float(formatter, *kind, *bound, unit.as_ref(), *rules),

            Type::Untagged(name) => write!(formatter, "union.@csup::untagged {name}"),
            Type::Struct(name) | Type::Union(name) | Type::New { name, .. } => {
                write!(formatter, "{name}")
            }
            Type::Task(result) => write!(formatter, "@concur::task<{result}>"),
            Type::Atomic(inner) => write!(formatter, "@concur::atomic<{inner}>"),
            Type::Lock(inner) => write!(formatter, "@concur::lock<{inner}>"),
            Type::Halves { item, mutable } => write!(
                formatter,
                "std::coll::halves{}<{item}>",
                if *mutable { ".mut" } else { "" }
            ),
            Type::ListView { item, mutable } => write!(
                formatter,
                "std::coll::list{}<{item}>",
                if *mutable { ".mut" } else { "" }
            ),
            Type::Holding {
                base,
                sources,
                fills,
            } => {
                let pairs: Vec<String> = sources
                    .iter()
                    .zip(fills)
                    .map(|(source, fill)| format!("`{source}`: {fill}"))
                    .collect();
                write!(formatter, "{base}<{}>", pairs.join(", "))
            }
            Type::List {
                item,
                min,
                max,
                len_type,
                grow,
            } => {
                write!(formatter, "std::coll::list<{item}>[")?;
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
                match grow {
                    Growth::Double => {}
                    Growth::Exact => write!(formatter, ", grow: exact")?,
                    Growth::By { count, unit } => write!(formatter, ", grow: by {count} {unit}")?,
                }
                write!(formatter, "]")
            }
            Type::Array {
                item,
                len,
                len_type,
            } => {
                write!(formatter, "std::coll::array<{item}>[len: {len} items")?;
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
                write!(
                    formatter,
                    "std::coll::map.{narrowing}<key.type: {key}, val.type: {value}>"
                )
            }
            Type::Failure { name, .. } => write!(formatter, "the failure `{name}`"),
            Type::Borrow {
                mutable,
                sources,
                inner,
            } => write_borrow(formatter, *mutable, sources, inner),
        }
    }
}

/// How a growable list grows when it's full: doubling, unless its type says `grow:`. (D385)
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Growth {
    Double,
    Exact,
    By { count: i128, unit: String },
}

/// The block a growable list holds: its allocator, how many items it has room for, and which of
/// an arena's blocks it is. (D322, D385, D389)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Heap {
    pub allocator: usize,
    pub capacity: usize,
    pub block: usize,
}

/// A view onto part of a list that lives elsewhere: half of a split, through which reads and
/// writes reach the list itself. (D382)
#[derive(Clone, Debug, PartialEq)]
pub struct Window {
    pub root: std::rc::Rc<std::cell::RefCell<Value>>,
    pub path: Vec<usize>,
    pub start: usize,
    pub len: usize,
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
    /// Lists and arrays look the same once running. A list that grows on the heap remembers
    /// its block once it first allocates: which allocator, by number, and its room. (D322, D385)
    List(Vec<Value>, Option<Heap>),
    Map,
    /// A C union's bytes, as many as the field written holds, least significant first, as on
    /// every target. (D398)
    Bytes(Vec<u8>),
    /// Half of a split. (D382)
    Window(Window),
}

impl Value {
    /// How a value shows in `when:`. Only the simple ones are shown.
    pub fn shown(&self) -> Option<String> {
        match self {
            Value::Bool(value) => Some(value.to_string()),
            Value::Integer(value) => Some(value.to_string()),
            Value::F32(value) => Some(show_float(FloatKind::F32, f64::from(*value))),
            Value::F64(value) => Some(show_float(FloatKind::F64, *value)),
            _ => None,
        }
    }
}

fn write_borrow(
    formatter: &mut fmt::Formatter<'_>,
    mutable: bool,
    sources: &Sources,
    inner: &Type,
) -> fmt::Result {
    write!(
        formatter,
        "{} from ",
        if mutable { "Lend.mut" } else { "Lend" }
    )?;
    match sources {
        Sources::Program => write!(formatter, "program")?,
        Sources::Named(name) => write!(formatter, "`{name}`")?,
        Sources::Variables(names) => {
            if let [only] = names.as_slice() {
                write!(formatter, "${only}")?;
            } else {
                let joined: Vec<String> = names.iter().map(|name| format!("${name}")).collect();
                write!(formatter, "<{}>", joined.join(", "))?;
            }
        }
    }
    write!(formatter, " {inner}")
}
