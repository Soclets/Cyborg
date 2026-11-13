//! The `okay` command (D222, D224): `okay comp --FinalSafe main.cy` compiles, and `okay run --Working main.cy`
//! compiles and runs.

#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![warn(clippy::pedantic)]

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use okay::build::{self, Outcome};
use okay::target::{Mode, Optimization, Target};

const USAGE: &str = "usage: okay comp|run --Working|--FinalSafe|--FinalPerf|--FinalShit|--FinalSmall|--Literal [--target=…] [--max_cores=…] file.cy";

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
    let mut optimization = None;
    let mut target = Target::host();
    let mut file = None;
    let mut cores = None;
    let mut machine_cpu: Option<String> = None;
    let mut profile: Option<String> = None;
    for argument in &arguments[1..] {
        if let Some(chosen) = Mode::from_flag(argument) {
            if mode.is_some() {
                eprintln!("okay: choose one build mode");
                return ExitCode::from(2);
            }
            mode = Some(chosen);
        } else if let Some(level) = Optimization::from_flag(argument) {
            optimization = Some(level);
        } else if argument.starts_with("--opt=") {
            eprintln!("okay: `{argument}` isn't a level; `--opt=` takes LLVM's: O0, O1, O2, O3, Os or Oz (D288)");
            return ExitCode::from(2);
        } else if argument == "--mode" {
            eprintln!("okay: the build mode is a flag of its own, as in `--FinalSafe` (D224)");
            return ExitCode::from(2);
        } else if let Some(count) = argument.strip_prefix("--max_cores=") {
            // trace: D224
            let available = std::thread::available_parallelism().map_or(1, std::num::NonZero::get);
            cores = match count {
                "max" => Some(available),
                "min" => Some(1),
                _ => match count.parse::<usize>() {
                    Ok(number) if number > 0 => Some(number),
                    _ => {
                        eprintln!("okay: `--max_cores=` takes a number above 0, `max` or `min`, as in `--max_cores=4`");
                        return ExitCode::from(2);
                    }
                },
            };
        } else if let Some(name) = argument.strip_prefix("--machine.cpu=") {
            // trace: D365
            machine_cpu = Some(name.to_string());
        } else if argument.starts_with("--machine.gpu=") {
            eprintln!(
                "okay: `--machine.gpu=` goes to the limb that gives a program its GPU code, and no limb here gives any (D365)"
            );
            return ExitCode::from(2);
        } else if argument == "--profile.record" || argument.starts_with("--profile.use=") {
            profile = Some(argument.clone());
        } else if let Some(name) = argument.strip_prefix("--target=") {
            if let Some(chosen) = Target::from_name(name) {
                target = chosen;
                okay::target::set_named();
            } else {
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
    let mode = match (mode, optimization) {
        (Mode::Working(_), Some(level)) => Mode::Working(level),
        (Mode::Working(_), None) => {
            eprintln!(
                "okay: `--Working` says how much it optimizes: `--opt=O0` for debugging, or `--opt=O1` for quick runs (D288)"
            );
            return ExitCode::from(2);
        }
        (other, Some(_)) => {
            eprintln!(
                "okay: `--opt=` goes only with `--Working`, since `--{}` optimizes as it says (D288)",
                other.name()
            );
            return ExitCode::from(2);
        }
        (other, None) => other,
    };
    // trace: D363, D365
    if mode == Mode::Literal && machine_cpu.is_some() {
        eprintln!("okay: `--machine.cpu=` can't go with `--Literal`, which builds exactly what's written (D282, D365)");
        return ExitCode::from(2);
    }
    if let Some(flag) = &profile {
        if matches!(mode, Mode::Literal | Mode::Working(Optimization::O0)) {
            eprintln!(
                "okay: `{flag}` tunes an optimizing build, so it can't go with `--{}` (D282, D363)",
                if mode == Mode::Literal { "Literal" } else { "Working --opt=O0" }
            );
            return ExitCode::from(2);
        }
        eprintln!(
            "NotBuiltYet: `{flag}` needs LLVM's profile counters and their runtime, which Lyftar doesn't give yet (D286, D363)"
        );
        return ExitCode::from(1);
    }
    if let Some(name) = &machine_cpu {
        okay::target::set_machine_cpu(name);
    }
    let Some(file) = file else {
        eprintln!("{USAGE}");
        return ExitCode::from(2);
    };
    let output = output_path(&file, target, run);
    // `--max_cores=` is `max` when left out (standard 10.2).
    let cores = cores.unwrap_or_else(|| std::thread::available_parallelism().map_or(1, std::num::NonZero::get));
    match build::build(&file, target, mode, cores, &output) {
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
