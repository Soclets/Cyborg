//! Runs every program in the repository's `tests/` folder, and in each folder inside it, and
//! compares its output byte for byte. `name.cy` is the program, `name.out` its exact standard
//! output, and `name.err` its exact error. A missing file means that stream stays empty, and an
//! `.err` means the oracle must exit with 1, unless `name.exit` gives the code, as for a program
//! that prints to stderr and ends well. (D195) `name.target` names the target a test's answer
//! depends on, passed as `--target=`. (D387, D389) A folder is one project with one manifest, so a
//! test of the manifest itself has a folder of its own. (D183)
#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![warn(clippy::pedantic)]

use std::fmt::Write as _;
use std::path::Path;
use std::process::Command;

fn read_or_empty(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap_or_default()
}

fn run_one(folder: &Path, stem: &str) -> Result<(), String> {
    let source = format!("{stem}.cy");
    // A test whose answer depends on its target names it in `name.target`. (D387, D389)
    let target = read_or_empty(&folder.join(format!("{stem}.target")));
    let mut command = Command::new(env!("CARGO_BIN_EXE_oracle"));
    if !target.trim().is_empty() {
        command.arg(format!("--target={}", target.trim()));
    }
    let output = command
        .arg(&source)
        .current_dir(folder)
        .output()
        .map_err(|error| format!("{stem}: couldn't start the oracle: {error}"))?;
    let got_out = String::from_utf8_lossy(&output.stdout);
    let got_err = String::from_utf8_lossy(&output.stderr);
    let want_out = read_or_empty(&folder.join(format!("{stem}.out")));
    let want_err = read_or_empty(&folder.join(format!("{stem}.err")));
    let mut problems = String::new();
    if got_out != want_out {
        let _ = write!(
            problems,
            "\n--- wanted stdout\n{want_out}--- got stdout\n{got_out}"
        );
    }
    if got_err != want_err {
        let _ = write!(
            problems,
            "\n--- wanted stderr\n{want_err}--- got stderr\n{got_err}"
        );
    }
    let written_code = read_or_empty(&folder.join(format!("{stem}.exit")));
    let want_code = match written_code.trim().parse::<i32>() {
        Ok(code) => code,
        Err(_) => i32::from(!want_err.is_empty()),
    };
    if output.status.code() != Some(want_code) {
        let _ = write!(
            problems,
            "\n--- wanted exit code {want_code}, got {:?}",
            output.status.code()
        );
    }
    if problems.is_empty() {
        Ok(())
    } else {
        Err(format!("{stem}:{problems}"))
    }
}

/// The programs in one folder, by name without `.cy`, in order.
fn programs_in(folder: &Path) -> Vec<String> {
    let mut stems: Vec<String> = std::fs::read_dir(folder)
        .map(|entries| {
            entries
                .filter_map(Result::ok)
                .filter_map(|entry| entry.file_name().into_string().ok())
                .filter_map(|name| name.strip_suffix(".cy").map(str::to_string))
                .collect()
        })
        .unwrap_or_default();
    stems.sort();
    stems
}

#[test]
fn every_program_matches_byte_for_byte() {
    let tests = Path::new(env!("CARGO_MANIFEST_DIR")).join("../tests");
    let mut folders = vec![tests.clone()];
    let mut inside: Vec<_> = std::fs::read_dir(&tests)
        .map(|entries| {
            entries
                .filter_map(Result::ok)
                .map(|entry| entry.path())
                .filter(|path| path.is_dir())
                .collect()
        })
        .unwrap_or_default();
    inside.sort();
    folders.extend(inside);
    let mut count = 0;
    let mut failures: Vec<String> = Vec::new();
    for folder in &folders {
        let shown = folder
            .strip_prefix(&tests)
            .map(|relative| relative.to_string_lossy().into_owned())
            .unwrap_or_default();
        for stem in programs_in(folder) {
            count += 1;
            if let Err(problem) = run_one(folder, &stem) {
                failures.push(if shown.is_empty() {
                    problem
                } else {
                    format!("{shown}/{problem}")
                });
            }
        }
    }
    assert!(count > 0, "no programs found in {}", tests.display());
    assert!(
        failures.is_empty(),
        "{} of {} programs differ:\n\n{}",
        failures.len(),
        count,
        failures.join("\n\n")
    );
}
