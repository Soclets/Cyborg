//! The live comparison: the oracle and `OKAY\n` run as black boxes on the same program, in
//! every build mode, and any difference is a bug in one of them, which the standard decides
//! (rule 3, D313).

use std::path::{Path, PathBuf};
use std::process::Command;

/// What a run gave: its standard output, its standard error and its exit code.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Outcome {
    pub out: String,
    pub error: String,
    pub code: Option<i32>,
}

impl Outcome {
    /// The oracle stopped on something undecided, so the program is out of scope.
    pub fn undecided(&self) -> bool {
        self.error.starts_with("NotDesignedYet:") || self.error.contains("\nNotDesignedYet:")
    }

    /// The oracle stopped on something decided that it doesn't handle yet.
    pub fn unbuilt(&self) -> bool {
        self.error.starts_with("NotBuiltYet:") || self.error.contains("\nNotBuiltYet:")
    }

    /// The program ran into a check, which only some modes keep (10.3, 10.4).
    pub fn checked(&self) -> bool {
        self.error.starts_with("check failed:") || self.error.contains("\ncheck failed:")
    }
}

/// A build mode and how `okay run` is told it (10.1, D288).
#[derive(Clone, Debug)]
pub struct Mode {
    pub flags: Vec<String>,
    /// Whether the mode keeps every check the oracle makes, so a failed check must match.
    pub keeps_checks: bool,
}

impl Mode {
    pub fn name(&self) -> String {
        self.flags.join(" ")
    }
}

/// Every mode: `Working` at each of LLVM's levels, then the others. `FinalPerf`, `FinalShit`
/// and `FinalSmall` drop or change checks, so only programs that pass every check must match
/// there. `Literal` keeps `FinalSafe`'s checks (D282).
pub fn every_mode() -> Vec<Mode> {
    let mut modes = Vec::new();
    for level in ["O0", "O1", "O2", "O3", "Os", "Oz"] {
        modes.push(Mode {
            flags: vec!["--Working".to_string(), format!("--opt={level}")],
            keeps_checks: true,
        });
    }
    for (flag, keeps_checks) in [
        ("--FinalSafe", true),
        ("--FinalPerf", false),
        ("--FinalShit", false),
        ("--FinalSmall", false),
        ("--Literal", true),
    ] {
        modes.push(Mode {
            flags: vec![flag.to_string()],
            keeps_checks,
        });
    }
    modes
}

fn run(command: &mut Command) -> Outcome {
    match command.output() {
        Ok(output) => Outcome {
            out: String::from_utf8_lossy(&output.stdout).into_owned(),
            error: String::from_utf8_lossy(&output.stderr).into_owned(),
            code: output.status.code(),
        },
        Err(problem) => Outcome {
            out: String::new(),
            error: format!("check: couldn't run it: {problem}"),
            code: None,
        },
    }
}

/// Runs a program in its own folder, so its manifest is found and messages name it the same.
fn in_folder(program: &Path) -> (PathBuf, String) {
    let folder = program
        .parent()
        .map_or_else(|| PathBuf::from("."), Path::to_path_buf);
    let name = program
        .file_name()
        .map_or_else(String::new, |name| name.to_string_lossy().into_owned());
    (folder, name)
}

pub fn run_oracle(oracle: &Path, program: &Path) -> Outcome {
    let (folder, name) = in_folder(program);
    run(Command::new(oracle).arg(&name).current_dir(folder))
}

pub fn run_okay(okay: &Path, mode: &Mode, program: &Path) -> Outcome {
    let (folder, name) = in_folder(program);
    run(Command::new(okay)
        .arg("run")
        .args(&mode.flags)
        .arg(&name)
        .current_dir(folder))
}

/// How one program compared.
pub enum Verdict {
    /// Every mode that had to match did.
    Same,
    /// The oracle says the program reaches something undecided, or not built, so it says
    /// nothing about the compiler.
    OutOfScope(&'static str),
    /// `f64.fast` only has to be close (rule 5, D163), which this doesn't judge.
    Close,
    /// These modes differ from the oracle.
    Different(Vec<(Mode, Outcome)>),
}

/// Compares a program in every mode that has to match the oracle.
pub fn compare(oracle: &Path, okay: &Path, modes: &[Mode], program: &Path) -> (Outcome, Verdict) {
    let expected = run_oracle(oracle, program);
    if expected.undecided() {
        return (expected, Verdict::OutOfScope("NotDesignedYet"));
    }
    if expected.unbuilt() {
        return (expected, Verdict::OutOfScope("NotBuiltYet"));
    }
    let text = std::fs::read_to_string(program).unwrap_or_default();
    if text.contains(".fast") {
        return (expected, Verdict::Close);
    }
    let mut different = Vec::new();
    for mode in modes {
        if expected.checked() && !mode.keeps_checks {
            continue;
        }
        let found = run_okay(okay, mode, program);
        if found != expected {
            different.push((mode.clone(), found));
        }
    }
    if different.is_empty() {
        (expected, Verdict::Same)
    } else {
        (expected, Verdict::Different(different))
    }
}
