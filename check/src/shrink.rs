//! Shrinking a difference to the smallest program that still shows it (rule 3, D313): drop
//! statements and functions, cut loops down and flatten `if`s, keeping each cut only when the
//! oracle's kind of outcome stays the same and the compiler still disagrees with it.

use std::path::Path;

use crate::compare::{run_okay, run_oracle, Mode, Outcome};
use crate::tree::{print, Expression, Handling, Program, Statement};

/// What kind of outcome a run had, which a cut has to keep.
fn kind_of(outcome: &Outcome) -> u8 {
    if outcome.undecided() || outcome.unbuilt() {
        0
    } else if outcome.checked() {
        1
    } else if outcome.code == Some(0) {
        2
    } else {
        3
    }
}

/// Whether this program still shows a difference of the same kind.
fn still_differs(
    oracle: &Path,
    okay: &Path,
    mode: &Mode,
    path: &Path,
    program: &Program,
    kind: u8,
) -> bool {
    if std::fs::write(path, print(program)).is_err() {
        return false;
    }
    let expected = run_oracle(oracle, path);
    if kind_of(&expected) != kind || kind == 0 {
        return false;
    }
    run_okay(okay, mode, path) != expected
}

/// Every smaller program one cut away, biggest cuts first.
fn cuts(program: &Program) -> Vec<Program> {
    let mut found = Vec::new();
    for index in 0..program.functions.len() {
        let mut smaller = program.clone();
        smaller.functions.remove(index);
        found.push(smaller);
    }
    // A declaration nothing uses goes; one in use leaves a refusal, which isn't kept.
    for index in 0..program.unions.len() {
        let mut smaller = program.clone();
        smaller.unions.remove(index);
        found.push(smaller);
    }
    for index in 0..program.structs.len() {
        let mut smaller = program.clone();
        smaller.structs.remove(index);
        found.push(smaller);
    }
    for (index, function) in program.functions.iter().enumerate() {
        if function.requires.is_some() || function.ensures.is_some() {
            let mut smaller = program.clone();
            if let Some(function) = smaller.functions.get_mut(index) {
                function.requires = None;
                function.ensures = None;
            }
            found.push(smaller);
        }
        for returned in expression_cuts(&function.returned) {
            let mut smaller = program.clone();
            if let Some(function) = smaller.functions.get_mut(index) {
                function.returned = returned;
            }
            found.push(smaller);
        }
    }
    for (index, function) in program.functions.iter().enumerate() {
        for body in block_cuts(&function.body) {
            let mut smaller = program.clone();
            if let Some(function) = smaller.functions.get_mut(index) {
                function.body = body;
            }
            found.push(smaller);
        }
    }
    for main in block_cuts(&program.main) {
        let mut smaller = program.clone();
        smaller.main = main;
        found.push(smaller);
    }
    // With every operation grouped, a difference that came from reading 3.9's table goes.
    if program.fewer_brackets {
        let mut grouped = program.clone();
        grouped.fewer_brackets = false;
        found.push(grouped);
    }
    found
}

/// A block one cut smaller: one statement gone, a loop shorter, or an `if` flattened.
fn block_cuts(block: &[Statement]) -> Vec<Vec<Statement>> {
    let mut found = Vec::new();
    for index in 0..block.len() {
        let mut without = block.to_vec();
        without.remove(index);
        found.push(without);
    }
    for (index, statement) in block.iter().enumerate() {
        let mut replace = |replacements: Vec<Statement>| {
            let mut changed = block[..index].to_vec();
            changed.extend(replacements);
            changed.extend_from_slice(&block[index + 1..]);
            found.push(changed);
        };
        match statement {
            Statement::If {
                test,
                then,
                otherwise,
            } => {
                replace(then.clone());
                replace(otherwise.clone());
                for smaller in block_cuts(then) {
                    replace(vec![Statement::If {
                        test: test.clone(),
                        then: smaller,
                        otherwise: otherwise.clone(),
                    }]);
                }
                for smaller in block_cuts(otherwise) {
                    replace(vec![Statement::If {
                        test: test.clone(),
                        then: then.clone(),
                        otherwise: smaller,
                    }]);
                }
            }
            Statement::While {
                counter,
                rounds,
                body,
            } => {
                if *rounds > 1 {
                    replace(vec![Statement::While {
                        counter: counter.clone(),
                        rounds: 1,
                        body: body.clone(),
                    }]);
                }
                for smaller in block_cuts(body) {
                    replace(vec![Statement::While {
                        counter: counter.clone(),
                        rounds: *rounds,
                        body: smaller,
                    }]);
                }
            }
            Statement::ForEach { body, .. } | Statement::For { body, .. } => {
                for smaller in block_cuts(body) {
                    let mut copy = statement.clone();
                    if let Statement::ForEach { body, .. } | Statement::For { body, .. } = &mut copy
                    {
                        *body = smaller;
                    }
                    replace(vec![copy]);
                }
            }
            _ => {}
        }
        // A statement with its expression made smaller.
        for smaller in statement_cuts(statement) {
            replace(vec![smaller]);
        }
    }
    found
}

/// A statement one expression cut smaller.
fn statement_cuts(statement: &Statement) -> Vec<Statement> {
    let mut found = Vec::new();
    match statement {
        Statement::Let {
            name,
            value_type,
            value,
        } => {
            for value in expression_cuts(value) {
                found.push(Statement::Let {
                    name: name.clone(),
                    value_type: *value_type,
                    value,
                });
            }
        }
        Statement::Change { name, value } => {
            for value in expression_cuts(value) {
                found.push(Statement::Change {
                    name: name.clone(),
                    value,
                });
            }
        }
        Statement::Print(value) => {
            for value in expression_cuts(value) {
                found.push(Statement::Print(value));
            }
        }
        Statement::If {
            test,
            then,
            otherwise,
        } => {
            for test in expression_cuts(test) {
                found.push(Statement::If {
                    test,
                    then: then.clone(),
                    otherwise: otherwise.clone(),
                });
            }
        }
        _ => {}
    }
    found
}

/// An expression one step smaller: an operation replaced by one of its sides, a call's
/// handling dropped, or a part inside made smaller. A side of another type leaves a refusal,
/// which isn't kept.
fn expression_cuts(value: &Expression) -> Vec<Expression> {
    let mut found = Vec::new();
    match value {
        Expression::Binary {
            operator,
            left,
            right,
        } => {
            found.push((**left).clone());
            found.push((**right).clone());
            for smaller in expression_cuts(left) {
                found.push(Expression::Binary {
                    operator: *operator,
                    left: Box::new(smaller),
                    right: right.clone(),
                });
            }
            for smaller in expression_cuts(right) {
                found.push(Expression::Binary {
                    operator: *operator,
                    left: left.clone(),
                    right: Box::new(smaller),
                });
            }
        }
        Expression::Compare {
            comparison,
            left,
            right,
        } => {
            for smaller in expression_cuts(right) {
                found.push(Expression::Compare {
                    comparison: *comparison,
                    left: left.clone(),
                    right: Box::new(smaller),
                });
            }
        }
        Expression::Logic {
            and,
            both,
            left,
            right,
        } => {
            found.push((**left).clone());
            found.push((**right).clone());
            for smaller in expression_cuts(left) {
                found.push(Expression::Logic {
                    and: *and,
                    both: *both,
                    left: Box::new(smaller),
                    right: right.clone(),
                });
            }
        }
        Expression::Not(inner) | Expression::Negate(inner) => {
            found.push((**inner).clone());
        }
        Expression::Call {
            function,
            arguments,
            handling,
        } => {
            if !matches!(handling, Handling::Crash) {
                found.push(Expression::Call {
                    function: function.clone(),
                    arguments: arguments.clone(),
                    handling: Handling::Crash,
                });
            }
            for (index, (label, argument)) in arguments.iter().enumerate() {
                for smaller in expression_cuts(argument) {
                    let mut arguments = arguments.clone();
                    if let Some(slot) = arguments.get_mut(index) {
                        *slot = (label.clone(), smaller);
                    }
                    found.push(Expression::Call {
                        function: function.clone(),
                        arguments,
                        handling: handling.clone(),
                    });
                }
            }
        }
        _ => {}
    }
    found
}

/// The smallest program found that still differs, written to `path`.
pub fn shrink(oracle: &Path, okay: &Path, mode: &Mode, path: &Path, program: &Program) -> Program {
    let _ = std::fs::write(path, print(program));
    let kind = kind_of(&run_oracle(oracle, path));
    let mut best = program.clone();
    'shrinking: loop {
        for smaller in cuts(&best) {
            if still_differs(oracle, okay, mode, path, &smaller, kind) {
                best = smaller;
                continue 'shrinking;
            }
        }
        break;
    }
    let _ = std::fs::write(path, print(&best));
    best
}
