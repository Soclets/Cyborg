//! The Cyborg oracle: runs a program, or refuses it, by following the design directly. (D62)
#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![warn(clippy::pedantic)]

mod ast;
mod check;
mod ir;
mod report;
mod run;
mod syntax;
mod types;

use std::process::ExitCode;

use report::{Outcome, Source};

/// Deep but ordinary Cyborg recursion needs far more stack than the default, and running out would
/// crash the oracle, which rule 6 forbids.
const STACK_BYTES: usize = 2 * 1024 * 1024 * 1024;

fn run_program(source: &Source) -> Outcome<()> {
    let program = syntax::parse(&source.text)?;
    let code = check::check(&source.text, &program)?;
    run::check_compile_time(&code)?;
    run::run(&code)
}

fn run_and_report(source: &Source) -> ExitCode {
    match run_program(source) {
        Ok(()) => ExitCode::SUCCESS,
        Err(report) => {
            eprint!("{}", source.render(&report));
            ExitCode::FAILURE
        }
    }
}

fn main() -> ExitCode {
    let mut arguments = std::env::args().skip(1);
    let (Some(path), None) = (arguments.next(), arguments.next()) else {
        eprintln!("usage: oracle <program.cy>");
        return ExitCode::from(2);
    };
    // trace: D137
    if !path.ends_with(".cy") {
        eprintln!("oracle: {path} isn't a source file, which ends in `.cy` (D137)");
        return ExitCode::from(2);
    }
    let text = match std::fs::read_to_string(&path) {
        Ok(text) => text,
        Err(error) => {
            eprintln!("oracle: couldn't read {path}: {error}");
            return ExitCode::from(2);
        }
    };
    let source = Source { name: path, text };
    let worker = std::thread::Builder::new()
        .stack_size(STACK_BYTES)
        .spawn(move || run_and_report(&source));
    let joined = match worker {
        Ok(handle) => handle.join(),
        Err(error) => {
            eprintln!("oracle: couldn't start: {error}");
            return ExitCode::from(2);
        }
    };
    // trace: D65
    joined.unwrap_or_else(|_| {
        eprintln!("oracle: this is a bug in the oracle, not in your program.");
        eprintln!("Please report it with the program that caused it.");
        ExitCode::from(3)
    })
}
