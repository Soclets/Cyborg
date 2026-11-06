//! What the checker hands to the runner: every name resolved, every type settled, and every hidden
//! check written out as a node.

use crate::ast::{CompareOperator, Narrowing};
use crate::report::Span;
use crate::types::{FloatKind, IntegerKind, Value};

/// A variable and the fields to walk from it.
#[derive(Clone, Debug)]
pub struct Place {
    pub slot: usize,
    pub path: Vec<usize>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Arithmetic {
    Add,
    Subtract,
    Multiply,
    PlusWrap,
    Divide(Narrowing),
    Remainder(Narrowing),
    BitAnd,
    BitOr,
    BitXor,
    ShiftLeft,
    ShiftRight,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Number {
    Integer(IntegerKind),
    Float(FloatKind),
}

#[derive(Clone, Debug)]
pub struct Node {
    pub span: Span,
    pub kind: NodeKind,
}

#[derive(Clone, Debug)]
pub enum Argument {
    /// A fresh or copied value, which the callee owns.
    Value(Node),
    /// `Lend` or `Lend.mut`: the callee works on the caller's own value.
    Alias(Place),
}

#[derive(Clone, Debug)]
pub enum NodeKind {
    Constant(Value),
    Read(Place),
    Global(usize),
    Field(Box<Node>, usize),
    Struct(Vec<(usize, Node)>),
    Case(usize, Vec<(usize, Node)>),
    Items(Vec<Node>),
    All(Box<Node>, usize),
    Arithmetic {
        operator: Arithmetic,
        number: Number,
        left: Box<Node>,
        right: Box<Node>,
    },
    BitNot(IntegerKind, Box<Node>),
    And(Box<Node>, Box<Node>),
    Or(Box<Node>, Box<Node>),
    AndBoth(Box<Node>, Box<Node>),
    OrBoth(Box<Node>, Box<Node>),
    Negate(Number, Box<Node>),
    Not(Box<Node>),
    Compare {
        first: Box<Node>,
        rest: Vec<(CompareOperator, Node)>,
    },
    /// `arguments` are in the order written, and each says which parameter it fills.
    Call {
        function: usize,
        arguments: Vec<(usize, Argument)>,
    },
    Println(Vec<Node>),
    Append {
        receiver: Place,
        shown: String,
        item: Box<Node>,
        max: Option<i128>,
    },
    Length(Box<Node>),
    IntegerToStr(Box<Node>),
    /// A failure with nothing handling it stops the program. (D96)
    Crash(Box<Node>),
    Fallback {
        value: Box<Node>,
        fallback: Box<Node>,
    },
    Handle {
        value: Box<Node>,
        arms: Vec<(String, Vec<Instruction>)>,
    },
    /// A plain `=`, `change` or argument checks a bound the compiler can't see fits. (D20)
    Fit {
        value: Box<Node>,
        low: i128,
        high: i128,
        target: String,
        shown: Option<String>,
    },
    /// The name is `void` here too, so `?? void` has something to give back.
    Void,
}

#[derive(Clone, Debug)]
pub struct Instruction {
    pub span: Span,
    pub kind: InstructionKind,
}

#[derive(Clone, Debug)]
pub enum InstructionKind {
    Let {
        slot: usize,
        value: Node,
    },
    /// A stored borrow names its source instead of holding a value of its own.
    LetAlias {
        slot: usize,
        place: Place,
    },
    Set {
        place: Place,
        value: Node,
    },
    If {
        condition: Node,
        body: Vec<Instruction>,
    },
    TestRange {
        subject: Node,
        low: i128,
        high: i128,
        body: Vec<Instruction>,
    },
    TestCase {
        subject: Node,
        case: usize,
        body: Vec<Instruction>,
    },
    Match {
        subject: Node,
        arms: Vec<(usize, Vec<Instruction>)>,
    },
    For {
        slot: usize,
        collection: Node,
        body: Vec<Instruction>,
    },
    Return(Node),
    Fail {
        name: String,
        shown: String,
        data: Vec<Node>,
    },
    Evaluate(Node),
    Block(Vec<Instruction>),
}

#[derive(Clone, Debug)]
pub struct Contract {
    pub condition: Node,
    /// Values for `when:`, read only once the contract has come out false.
    pub shown: Vec<(String, Node)>,
}

#[derive(Clone, Debug)]
pub struct Function {
    pub name: String,
    /// Each parameter's slot is its position, so `slot_count` starts past them.
    pub parameter_count: usize,
    pub slot_count: usize,
    pub result_slot: usize,
    pub requires: Vec<Contract>,
    pub ensures: Vec<Contract>,
    pub body: Vec<Instruction>,
}

#[derive(Clone, Debug)]
pub struct Code {
    pub main: usize,
    pub functions: Vec<Function>,
    /// In source order, so one can read an earlier one.
    pub globals: Vec<Node>,
    /// Everything `bake` and local `const` need worked out while building.
    pub compile_time: Vec<Node>,
}
