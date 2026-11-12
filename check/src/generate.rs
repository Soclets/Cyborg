//! Random programs, from the syntax tree in `tree.rs`. Every program is valid by the standard's
//! rules as written: typed `let`s, `=` for values, words for operators, spacing as 1.14 says,
//! loops that always end, and calls only to functions declared earlier, so nothing recurses.
//! What it doesn't generate is listed in `coverage.rs`, since all passing doesn't mean covered.

use crate::random::Random;
use crate::tree::{
    Comparison, Expression, FailureDecl, Filling, Function, Handling, Kind, Operator, Position,
    Program, Statement, Type,
};

/// How big a program gets.
pub struct Limits {
    pub functions: i128,
    pub statements: i128,
    pub depth: usize,
}

pub const LIMITS: Limits = Limits {
    functions: 3,
    statements: 8,
    depth: 3,
};

struct Scope {
    variables: Vec<(String, Type)>,
    /// Arrays, with their item type and length.
    arrays: Vec<(String, Kind, i128)>,
}

struct Generator<'random> {
    random: &'random mut Random,
    next_name: usize,
    /// The failures the function being made lists, which `?` adds to; none in `main`.
    failing: Option<Vec<FailureDecl>>,
    functions: Vec<Function>,
}

/// A program from a seed: the same seed always gives the same program.
pub fn program(seed: u64) -> Program {
    let mut random = Random::new(seed);
    let mut generator = Generator {
        random: &mut random,
        next_name: 0,
        failing: None,
        functions: Vec::new(),
    };
    let count = generator.random.between(0, LIMITS.functions);
    for _ in 0..count {
        let function = generator.function();
        generator.functions.push(function);
    }
    let mut scope = Scope {
        variables: Vec::new(),
        arrays: Vec::new(),
    };
    let mut main = generator.block(&mut scope, LIMITS.statements, 0, (true, false));
    // `main` ends by printing what it made, so a program's work shows in its output.
    for statement in main.clone() {
        if let Statement::Let {
            name,
            value_type: Type::Integer(_),
            ..
        }
        | Statement::LetOtherwise { name, .. } = statement
        {
            main.push(Statement::Print(Expression::Variable(name)));
        }
    }
    let fewer_brackets = generator.random.chance(50);
    Program {
        functions: generator.functions,
        main,
        fewer_brackets,
    }
}

impl Generator<'_> {
    /// A comparison chain of three values, all going one way, like `$x below 5 at_most 9`, so
    /// it reads as the range it looks like (3.9). The first is a variable, so each value after
    /// it has a typed side to take its type from.
    fn chain(&mut self, scope: &Scope, depth: usize) -> Option<Expression> {
        let integers: Vec<(String, Type)> = scope
            .variables
            .iter()
            .filter(|(_, found)| matches!(found, Type::Integer(_)))
            .cloned()
            .collect();
        let (name, Type::Integer(kind)) = self.random.pick(&integers).cloned()? else {
            return None;
        };
        let directions: [&[Comparison]; 3] = [
            &[Comparison::Below, Comparison::AtMost],
            &[Comparison::Above, Comparison::AtLeast],
            &[Comparison::Equal],
        ];
        let direction = *self.random.pick(&directions)?;
        let comparisons = vec![*self.random.pick(direction)?, *self.random.pick(direction)?];
        let middle = self.integer(scope, kind, depth - 1);
        let last = self.integer(scope, kind, depth - 1);
        Some(Expression::Chain {
            values: vec![Expression::Variable(name), middle, last],
            comparisons,
        })
    }

    fn name(&mut self, prefix: &str) -> String {
        self.next_name += 1;
        format!("{prefix}{}", self.next_name)
    }

    fn integer_type(&mut self) -> Type {
        Type::Integer(*self.random.pick(&Kind::ALL).unwrap_or(&Kind::I32))
    }

    fn function(&mut self) -> Function {
        let name = self.name("f");
        let count = self.random.between(1, 3);
        let mut scope = Scope {
            variables: Vec::new(),
            arrays: Vec::new(),
        };
        let mut parameters = Vec::new();
        for _ in 0..count {
            let parameter = self.name("p");
            let parameter_type = self.integer_type();
            scope.variables.push((parameter.clone(), parameter_type));
            parameters.push((parameter, parameter_type));
        }
        let result = self.integer_type();
        // Some functions can fail, each with a test that decides when (8.1, 8.2).
        let mut own = Vec::new();
        let mut body = Vec::new();
        if self.random.chance(40) {
            for _ in 0..self.random.between(1, 2) {
                let failure = self.name("bad");
                let carried = if self.random.chance(50) {
                    self.random.pick(&parameters).cloned()
                } else {
                    None
                };
                let data = carried.as_ref().and_then(|(_, found)| match found {
                    Type::Integer(kind) => Some(*kind),
                    Type::Bool => None,
                });
                let test = self.expression(&scope, Type::Bool, LIMITS.depth);
                body.push(Statement::If {
                    test,
                    then: vec![Statement::Fail {
                        name: failure.clone(),
                        data: data.and(carried.map(|(name, _)| Expression::Variable(name))),
                    }],
                    otherwise: Vec::new(),
                });
                own.push(FailureDecl {
                    name: failure,
                    data,
                });
            }
        }
        self.failing = Some(own);
        body.extend(self.block(&mut scope, 3, 1, (false, false)));
        let returned = self.expression(&scope, result, LIMITS.depth);
        let failures = self.failing.take().unwrap_or_default();
        Function {
            name,
            parameters,
            result,
            failures,
            body,
            returned,
        }
    }

    /// A block's statements. Only `main` prints, and only a loop's blocks `break`.
    fn block(
        &mut self,
        scope: &mut Scope,
        most: i128,
        depth: usize,
        (prints, in_loop): (bool, bool),
    ) -> Vec<Statement> {
        let declared_before = scope.variables.len();
        let arrays_before = scope.arrays.len();
        let count = self.random.between(1, most);
        let mut statements = Vec::new();
        for _ in 0..count {
            statements.push(self.statement(scope, depth, (prints, in_loop)));
        }
        // What a block declares ends with it.
        scope.variables.truncate(declared_before);
        scope.arrays.truncate(arrays_before);
        statements
    }

    #[allow(clippy::too_many_lines)] // One arm per kind of statement, read top to bottom.
    fn statement(
        &mut self,
        scope: &mut Scope,
        depth: usize,
        (prints, in_loop): (bool, bool),
    ) -> Statement {
        let roll = self.random.between(0, 10);
        let integers: Vec<String> = scope
            .variables
            .iter()
            // A loop's counter, `$i…`, and a plain parameter, `$p…`, can't change (D184).
            .filter(|(name, value_type)| {
                matches!(value_type, Type::Integer(_))
                    && !name.starts_with('i')
                    && !name.starts_with('p')
            })
            .map(|(name, _)| name.clone())
            .collect();
        match roll {
            0..=3 => {
                if self.random.chance(25) {
                    let kind = *self.random.pick(&Kind::ALL).unwrap_or(&Kind::I32);
                    if let Some(statement) = self.let_otherwise(scope, kind) {
                        return statement;
                    }
                }
                let value_type = if self.random.chance(20) {
                    Type::Bool
                } else {
                    self.integer_type()
                };
                let value = self.expression(scope, value_type, LIMITS.depth);
                let name = self.name("v");
                scope.variables.push((name.clone(), value_type));
                Statement::Let {
                    name,
                    value_type,
                    value,
                }
            }
            4 | 5 if !integers.is_empty() => {
                let name = self.random.pick(&integers).cloned().unwrap_or_default();
                let value_type = scope
                    .variables
                    .iter()
                    .find(|(found, _)| *found == name)
                    .map_or(Type::Integer(Kind::I32), |(_, found)| *found);
                let value = self.expression(scope, value_type, LIMITS.depth);
                Statement::Change { name, value }
            }
            6 if depth < 2 => {
                let test = self.expression(scope, Type::Bool, LIMITS.depth);
                let mut then = self.block(scope, 3, depth + 1, (prints, in_loop));
                // A `break` ends a block, so nothing after it goes unreached (4.8).
                if in_loop && self.random.chance(30) {
                    then.push(Statement::Break);
                }
                let otherwise = if self.random.chance(50) {
                    self.block(scope, 3, depth + 1, (prints, in_loop))
                } else {
                    Vec::new()
                };
                Statement::If {
                    test,
                    then,
                    otherwise,
                }
            }
            7 if depth < 2 => {
                let counter = self.name("i");
                let rounds = self.random.between(0, 5);
                scope
                    .variables
                    .push((counter.clone(), Type::Integer(Kind::I32)));
                let body = self.block(scope, 3, depth + 1, (prints, true));
                Statement::While {
                    counter,
                    rounds,
                    body,
                }
            }
            8 if depth < 2 => self.range_loop(scope, depth, prints),
            9 if self.random.chance(50) => self.array_statement(scope),
            _ if prints => {
                // A print's value is an integer variable, so its type is never left for a
                // literal to guess, and how a `bool` prints isn't decided (12.3).
                let printable: Vec<String> = scope
                    .variables
                    .iter()
                    .filter(|(_, value_type)| matches!(value_type, Type::Integer(_)))
                    .map(|(name, _)| name.clone())
                    .collect();
                if let Some(name) = self.random.pick(&printable).cloned() {
                    Statement::Print(Expression::Variable(name))
                } else {
                    {
                        let value_type = self.integer_type();
                        let value = self.expression(scope, value_type, LIMITS.depth);
                        let name = self.name("v");
                        scope.variables.push((name.clone(), value_type));
                        Statement::Let {
                            name,
                            value_type,
                            value,
                        }
                    }
                }
            }
            _ => {
                let value_type = self.integer_type();
                let value = self.expression(scope, value_type, LIMITS.depth);
                let name = self.name("v");
                scope.variables.push((name.clone(), value_type));
                Statement::Let {
                    name,
                    value_type,
                    value,
                }
            }
        }
    }

    /// A new array, or a change to an item of one in scope.
    fn array_statement(&mut self, scope: &mut Scope) -> Statement {
        if let Some((array, kind, len)) = self.random.pick(&scope.arrays).cloned() {
            if self.random.chance(60) {
                let position = self.position(scope, len);
                let value = self.expression(scope, Type::Integer(kind), LIMITS.depth);
                return Statement::ChangeItem {
                    array,
                    position,
                    value,
                };
            }
        }
        let kind = *self.random.pick(&Kind::ALL).unwrap_or(&Kind::I32);
        let len = self.random.between(1, 5);
        let filling = if self.random.chance(30) {
            Filling::All(self.literal(kind))
        } else {
            Filling::Items(
                (0..len)
                    .map(|_| self.expression(scope, Type::Integer(kind), LIMITS.depth))
                    .collect(),
            )
        };
        let name = self.name("a");
        scope.arrays.push((name.clone(), kind, len));
        Statement::LetArray {
            name,
            kind,
            len,
            filling,
        }
    }

    /// An item's place: a literal inside the array, counted either way, or now and then a
    /// variable's value, which may be outside it and is checked while running.
    fn position(&mut self, scope: &Scope, len: i128) -> Position {
        let integers: Vec<String> = scope
            .variables
            .iter()
            .filter(|(_, found)| matches!(found, Type::Integer(_)))
            .map(|(name, _)| name.clone())
            .collect();
        if self.random.chance(20) {
            if let Some(name) = self.random.pick(&integers).cloned() {
                return Position::PosOf(name);
            }
        }
        if self.random.chance(50) {
            Position::Pos(self.random.between(1, len))
        } else {
            Position::Offset(self.random.between(0, len - 1))
        }
    }

    /// `for` over a literal range that always reaches its end exactly, since a range that can't
    /// is refused and one that steps past it is a reading (4.5, 12.1). Its counter isn't used,
    /// since what type it has isn't stated.
    fn range_loop(&mut self, scope: &mut Scope, depth: usize, prints: bool) -> Statement {
        let counter = self.name("i");
        let from = self.random.between(-5, 5);
        let step = if self.random.chance(50) {
            None
        } else {
            Some(*self.random.pick(&[1, 2, 3, -1, -2]).unwrap_or(&1))
        };
        let inclusive = self.random.chance(50);
        let lowest = i128::from(!inclusive);
        let rounds = self.random.between(lowest, 4);
        let end = from + step.unwrap_or(1) * rounds;
        let body = self.block(scope, 3, depth + 1, (prints, true));
        Statement::For {
            counter,
            from,
            end,
            inclusive,
            step,
            body,
        }
    }

    /// A value of this type. Comparisons and calls only go where a typed value is around, so a
    /// literal never has to guess its type.
    fn expression(&mut self, scope: &Scope, value_type: Type, depth: usize) -> Expression {
        match value_type {
            Type::Bool => self.boolean(scope, depth),
            Type::Integer(kind) => self.integer(scope, kind, depth),
        }
    }

    fn variable_of(&mut self, scope: &Scope, value_type: Type) -> Option<Expression> {
        let found: Vec<&String> = scope
            .variables
            .iter()
            .filter(|(_, found)| *found == value_type)
            .map(|(name, _)| name)
            .collect();
        self.random
            .pick(&found)
            .map(|name| Expression::Variable((*name).clone()))
    }

    fn literal(&mut self, kind: Kind) -> Expression {
        let (low, high) = kind.range();
        let value = if self.random.chance(70) {
            self.random.between(low.max(-20), high.min(20))
        } else {
            self.random.between(low, high)
        };
        Expression::Integer(value)
    }

    fn integer(&mut self, scope: &Scope, kind: Kind, depth: usize) -> Expression {
        let roll = self.random.between(0, 9);
        if depth == 0 || roll < 3 {
            return match self.variable_of(scope, Type::Integer(kind)) {
                Some(variable) if self.random.chance(60) => variable,
                _ => self.literal(kind),
            };
        }
        if roll == 3 {
            if let Some(call) = self.call(scope, kind, depth) {
                return call;
            }
        }
        if roll == 5 {
            let arrays: Vec<(String, i128)> = scope
                .arrays
                .iter()
                .filter(|(_, found, _)| *found == kind)
                .map(|(name, _, len)| (name.clone(), *len))
                .collect();
            if let Some((array, len)) = self.random.pick(&arrays).cloned() {
                let position = self.position(scope, len);
                return Expression::Item { array, position };
            }
        }
        if roll == 4 && kind.range().0 < 0 {
            let inner = self.integer(scope, kind, depth - 1);
            return Expression::Negate(Box::new(inner));
        }
        let mut operator = *self.random.pick(&Operator::ALL).unwrap_or(&Operator::Add);
        // Whether plain `-` and `x` follow `+`'s overflow row isn't decided (12.3), so they're
        // usually their `.wrap` forms here, to keep more programs in scope.
        if self.random.chance(70) {
            operator = match operator {
                Operator::Subtract => Operator::MinusWrap,
                Operator::Multiply => Operator::TimesWrap,
                other => other,
            };
        }
        // Shifts stay on unsigned types, by a literal amount below the width, since shifting a
        // negative or by too much isn't decided (12.1, 12.3).
        if operator.is_shift() {
            if kind.range().0 < 0 {
                return self.literal(kind);
            }
            let left = self.integer(scope, kind, depth - 1);
            let amount = self.random.between(0, kind.bits() - 1);
            return Expression::Binary {
                operator,
                left: Box::new(left),
                right: Box::new(Expression::Integer(amount)),
            };
        }
        let left = self.integer(scope, kind, depth - 1);
        // Dividing by zero is a check every time, so a divisor is usually a literal that isn't
        // zero, to reach more of what follows.
        let right = if operator.divides() && self.random.chance(70) {
            let (_, high) = kind.range();
            Expression::Integer(self.random.between(1, high.min(20)))
        } else {
            self.integer(scope, kind, depth - 1)
        };
        Expression::Binary {
            operator,
            left: Box::new(left),
            right: Box::new(right),
        }
    }

    /// A call to a function made earlier with this result. One that can fail is handled with
    /// `?` where the function being made can pass its failures up, `??` and a value, or not at
    /// all, so a failure stops the program (8.4, 8.5, 8.9).
    fn call(&mut self, scope: &Scope, kind: Kind, depth: usize) -> Option<Expression> {
        let (function, arguments, failures) = self.plain_call(scope, kind, depth)?;
        let handling = if failures.is_empty() {
            Handling::Crash
        } else if self.failing.is_some() && self.random.chance(40) {
            if let Some(failing) = &mut self.failing {
                for failure in failures {
                    if !failing.iter().any(|found| found.name == failure.name) {
                        failing.push(failure);
                    }
                }
            }
            Handling::Pass
        } else if self.random.chance(70) {
            Handling::Fallback(Box::new(self.literal(kind)))
        } else {
            Handling::Crash
        };
        Some(Expression::Call {
            function,
            arguments,
            handling,
        })
    }

    /// A function made earlier with this result, its arguments, and what it can fail with.
    fn plain_call(
        &mut self,
        scope: &Scope,
        kind: Kind,
        depth: usize,
    ) -> Option<(String, Vec<Expression>, Vec<FailureDecl>)> {
        let candidates: Vec<(String, Vec<Type>, Vec<FailureDecl>)> = self
            .functions
            .iter()
            .filter(|function| function.result == Type::Integer(kind))
            .map(|function| {
                (
                    function.name.clone(),
                    function
                        .parameters
                        .iter()
                        .map(|(_, found)| *found)
                        .collect(),
                    function.failures.clone(),
                )
            })
            .collect();
        let (name, parameters, failures) = self.random.pick(&candidates)?.clone();
        let arguments = parameters
            .iter()
            .map(|parameter| self.expression(scope, *parameter, depth.saturating_sub(1)))
            .collect();
        Some((name, arguments, failures))
    }

    /// `let` from a call that can fail, with an `otherwise:` arm for each failure, each using
    /// a value `instead` (8.6).
    fn let_otherwise(&mut self, scope: &mut Scope, kind: Kind) -> Option<Statement> {
        let (function, arguments, failures) = self.plain_call(scope, kind, LIMITS.depth)?;
        if failures.is_empty() {
            return None;
        }
        let arms = failures
            .into_iter()
            .map(|failure| (failure.name, self.literal(kind)))
            .collect();
        let name = self.name("v");
        scope.variables.push((name.clone(), Type::Integer(kind)));
        Some(Statement::LetOtherwise {
            name,
            value_type: Type::Integer(kind),
            call: Expression::Call {
                function,
                arguments,
                handling: Handling::Crash,
            },
            arms,
        })
    }

    fn boolean(&mut self, scope: &Scope, depth: usize) -> Expression {
        let roll = self.random.between(0, 9);
        if depth == 0 || roll < 2 {
            return match self.variable_of(scope, Type::Bool) {
                Some(variable) => variable,
                None => Expression::Bool(self.random.chance(50)),
            };
        }
        if roll < 4 {
            let and = self.random.chance(50);
            let left = self.boolean(scope, depth - 1);
            let right = self.boolean(scope, depth - 1);
            return Expression::Logic {
                and,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        if roll == 4 {
            return Expression::Not(Box::new(self.boolean(scope, depth - 1)));
        }
        if roll == 5 {
            if let Some(chain) = self.chain(scope, depth) {
                return chain;
            }
        }
        // A comparison needs a typed side: a variable of some integer type.
        let integers: Vec<(String, Type)> = scope
            .variables
            .iter()
            .filter(|(_, found)| matches!(found, Type::Integer(_)))
            .cloned()
            .collect();
        let Some((name, Type::Integer(kind))) = self.random.pick(&integers).cloned() else {
            return Expression::Bool(self.random.chance(50));
        };
        let comparison = *self
            .random
            .pick(&Comparison::ALL)
            .unwrap_or(&Comparison::Equal);
        let right = self.integer(scope, kind, depth - 1);
        Expression::Compare {
            comparison,
            left: Box::new(Expression::Variable(name)),
            right: Box::new(right),
        }
    }
}
