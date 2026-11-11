//! The `okay` command (D222, D224): `okay comp --FinalSafe main.cy` compiles, and `okay run --Working main.cy`
//! compiles and runs.

#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![warn(clippy::pedantic)]

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use okay::build::{self, Outcome};
use okay::target::{Mode, Target};

const USAGE: &str =
    "usage: okay comp|run --Working|--FinalSafe|--FinalPerf|--FinalShit|--FinalSmall|--Literal [--target=…] file.cy";

fn main() -> ExitCode {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let Some(command) = arguments.first() else {
        eprintln!("{USAGE}");
        return ExitCode::from(2);
    };
    let run = match command.as_str() {
        "comp" => false,
        "run" => true,
        _ => {
            eprintln!("{USAGE}");
            return ExitCode::from(2);
        }
    };
    let mut mode = None;
    let mut target = Target::host();
    let mut file = None;
    for argument in &arguments[1..] {
        if let Some(chosen) = Mode::from_flag(argument) {
            if mode.is_some() {
                eprintln!("okay: choose one build mode");
                return ExitCode::from(2);
            }
            mode = Some(chosen);
        } else if argument == "--mode" {
            eprintln!("okay: the build mode is a flag of its own, as in `--FinalSafe` (D224)");
            return ExitCode::from(2);
        } else if let Some(name) = argument.strip_prefix("--target=") {
            if let Some(chosen) = Target::from_name(name) { target = chosen } else {
                eprintln!("okay: `{name}` isn't a target this compiler builds for");
                return ExitCode::from(2);
            }
        } else if argument.starts_with("--") {
            eprintln!("okay: `{argument}` isn't a flag");
            return ExitCode::from(2);
        } else {
            file = Some(PathBuf::from(argument));
        }
    }
    let Some(mode) = mode else {
        eprintln!("okay: there's no default build mode, so say which, as in `okay {command} --FinalSafe` (D26)");
        return ExitCode::from(2);
    };
    let Some(file) = file else {
        eprintln!("{USAGE}");
        return ExitCode::from(2);
    };
    let output = output_path(&file, target, run);
    match build::build(&file, target, mode, &output) {
        Outcome::Refused(text) => {
            eprint!("{text}");
            ExitCode::from(1)
        }
        Outcome::Failed(message) => {
            eprintln!("okay: the compiler itself failed: {message}");
            eprintln!(
                "This is a bug in the compiler, not your program. Please report it with the program that caused it."
            );
            ExitCode::from(3)
        }
        Outcome::Built => {
            if !run {
                return ExitCode::SUCCESS;
            }
            let status = std::process::Command::new(&output).status();
            let _ = std::fs::remove_file(&output);
            match status {
                Ok(status) => ExitCode::from(u8::try_from(status.code().unwrap_or(1)).unwrap_or(1)),
                Err(error) => {
                    eprintln!("okay: couldn't run the program: {error}");
                    ExitCode::from(3)
                }
            }
        }
    }
}

/// Where the program goes: next to the source for `comp`, and a temporary file for `run`.
fn output_path(file: &Path, target: Target, run: bool) -> PathBuf {
    let stem = file.file_stem().map_or_else(|| "program".into(), |stem| stem.to_string_lossy().into_owned());
    if run {
        std::env::temp_dir().join(format!("okay-run-{}-{stem}", std::process::id()))
    } else {
        let name = if target.has_os() { stem } else { format!("{stem}.elf") };
        file.with_file_name(name)
    }
}
