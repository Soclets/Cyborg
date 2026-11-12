//! The Cyborg oracle: runs a program, or refuses it, by following the design directly. (D62)
#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![warn(clippy::pedantic)]

mod ast;
mod cascade;
mod check;
mod ir;
mod manifest;
mod maths;
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

/// Builds a program, or gives every error building found, in the order they appear. `needs` is
/// what the manifest lists, when there is one. (D207, D183)
fn build(
    text: &str,
    needs: Option<&[String]>,
    target: Option<&'static str>,
) -> Result<ir::Code, Vec<Report>> {
    let (program, mut errors) = syntax::parse(text);
    let Some(program) = program else {
        return Err(check::in_order(errors));
    };
    let code = match check::check(text, &program, needs, target) {
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

/// Every program has a manifest next to it, `CyJet.siml`. Its errors show first, against it,
/// and then the program's. Without one, nothing checks what the program imports against it.
/// trace: D183, D229
/// The targets the oracle knows, by name, with how many bytes a pointer takes on each: an
/// operating system by the OS and the chip family, and bare metal by the board. (D387, D391)
/// trace: D387, D391
const TARGETS: &[(&str, u64)] = &[
    ("macos_aarch64", 8),
    ("linux_aarch64", 8),
    ("linux_x86_64", 8),
    ("arduino_uno_r4_wifi", 4),
];

fn run_and_report(
    source: &Source,
    manifest: Option<&Source>,
    (target, pointer_bytes): (Option<&'static str>, Option<u64>),
) -> ExitCode {
    let mut shown: Vec<String> = Vec::new();
    let needs = if let Some(manifest) = manifest {
        let (read, errors) = manifest::read(&manifest.text);
        let errors = check::in_order(errors);
        shown.extend(errors.iter().map(|report| manifest.render(report)));
        Some(read.needs)
    } else {
        shown.push(source.render(&Report::refused(
            report::Span::new(0, 0),
            "there's no `CyJet.siml` next to this program",
            "every program has a manifest, hello world included (D183, D229)",
        )));
        None
    };
    let needs = needs.as_deref();
    let errors = match build(&source.text, needs, target) {
        Ok(code) if shown.is_empty() => match run::run(&code, pointer_bytes) {
            Ok(()) => return ExitCode::SUCCESS,
            Err(report) => vec![*report],
        },
        Ok(_) => Vec::new(),
        // trace: D207
        Err(errors) => cascade::mark_cascades(&source.text, errors, |text| {
            build(text, needs, target).err().unwrap_or_default()
        }),
    };
    shown.extend(errors.iter().map(|report| source.render(report)));
    eprint!("{}", shown.join("\n"));
    ExitCode::FAILURE
}

/// The manifest next to a program, or nothing when there's none. Any other failure to read it
/// stops the oracle, as one reading the program does.
fn read_manifest(program: &str) -> Result<Option<Source>, String> {
    let folder = std::path::Path::new(program)
        .parent()
        .unwrap_or(std::path::Path::new(""));
    let path = folder.join("CyJet.siml");
    match std::fs::read_to_string(&path) {
        Ok(text) => Ok(Some(Source {
            name: path.to_string_lossy().into_owned(),
            text,
        })),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(format!("oracle: couldn't read {}: {error}", path.display())),
    }
}

fn main() -> ExitCode {
    // `--target=` names the target whose sizes a run uses, as `okay` takes it. (D387, D389)
    let mut target = None;
    let mut paths = Vec::new();
    for argument in std::env::args().skip(1) {
        match argument.strip_prefix("--target=") {
            Some(name) => target = Some(name.to_string()),
            None => paths.push(argument),
        }
    }
    let [path] = &paths[..] else {
        eprintln!("usage: oracle [--target=<target>] <program.cy>");
        return ExitCode::from(2);
    };
    let path = path.clone();
    let (target, pointer_bytes) = match &target {
        None => (None, None),
        Some(name) => {
            let Some((known, bytes)) = TARGETS.iter().find(|(known, _)| known == name) else {
                let known: Vec<&str> = TARGETS.iter().map(|(known, _)| *known).collect();
                eprintln!(
                    "oracle: `{name}` isn't a target the oracle knows; it knows {}",
                    known.join(", ")
                );
                return ExitCode::from(2);
            };
            (Some(*known), Some(*bytes))
        }
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
    let manifest = match read_manifest(&path) {
        Ok(manifest) => manifest,
        Err(problem) => {
            eprintln!("{problem}");
            return ExitCode::from(2);
        }
    };
    let source = Source { name: path, text };
    let worker = std::thread::Builder::new()
        .stack_size(STACK_BYTES)
        .spawn(move || run_and_report(&source, manifest.as_ref(), (target, pointer_bytes)));
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
