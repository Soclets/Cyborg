//! The Cyborg oracle: runs a program, or refuses it, by following the design directly. (D62)
#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![warn(clippy::pedantic)]

mod ast;
mod cascade;
mod check;
mod ir;
mod print;
mod report;
mod run;
mod syntax;
mod types;

use std::process::ExitCode;

use report::{Report, Source};

/// Deep but ordinary Cyborg recursion needs far more stack than the default, and running out would
/// crash the oracle, which rule 6 forbids.
const STACK_BYTES: usize = 2 * 1024 * 1024 * 1024;

/// Builds a program, or gives every error building found, in the order they appear. (D207)
fn build(text: &str) -> Result<ir::Code, Vec<Report>> {
    let (program, mut errors) = syntax::parse(text);
    let Some(program) = program else {
        return Err(check::in_order(errors));
    };
    let code = match check::check(text, &program) {
        Ok(code) if errors.is_empty() => code,
        Ok(_) => return Err(check::in_order(errors)),
        Err(more) => {
            errors.extend(more);
            return Err(check::in_order(errors));
        }
    };
    run::check_compile_time(&code).map_err(|report| vec![*report])?;
    Ok(code)
}

fn run_and_report(source: &Source) -> ExitCode {
    let errors = match build(&source.text) {
        Ok(code) => match run::run(&code) {
            Ok(()) => return ExitCode::SUCCESS,
            Err(report) => vec![*report],
        },
        // trace: D207
        Err(errors) => cascade::mark_cascades(&source.text, errors, |text| {
            build(text).err().unwrap_or_default()
        }),
    };
    let shown: Vec<String> = errors.iter().map(|report| source.render(report)).collect();
    eprint!("{}", shown.join("\n"));
    ExitCode::FAILURE
}

fn main() -> ExitCode {
    let mut arguments = std::env::args().skip(1);
    let (Some(path), None) = (arguments.next(), arguments.next()) else {
        eprintln!("usage: oracle <program.cy>");
        return ExitCode::from(2);
    };
    // trace: D137
    let extension = std::path::Path::new(&path).extension();
    if extension.is_none_or(|found| found != "cy") {
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
