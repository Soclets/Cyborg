//! Runs every program in the repository's `tests/` folder, and compares its output byte for byte.
//! `name.cy` is the program, `name.out` its exact standard output, and `name.err` its exact error.
//! A missing file means that stream stays empty, and an `.err` means the oracle must exit with 1.
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
    let output = Command::new(env!("CARGO_BIN_EXE_oracle"))
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
    let want_code = i32::from(!want_err.is_empty());
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

#[test]
fn every_program_matches_byte_for_byte() {
    let folder = Path::new(env!("CARGO_MANIFEST_DIR")).join("../tests");
    let mut stems: Vec<String> = std::fs::read_dir(&folder)
        .map(|entries| {
            entries
                .filter_map(Result::ok)
                .filter_map(|entry| entry.file_name().into_string().ok())
                .filter_map(|name| name.strip_suffix(".cy").map(str::to_string))
                .collect()
        })
        .unwrap_or_default();
    stems.sort();
    assert!(
        !stems.is_empty(),
        "no programs found in {}",
        folder.display()
    );
    let failures: Vec<String> = stems
        .iter()
        .filter_map(|stem| run_one(&folder, stem).err())
        .collect();
    assert!(
        failures.is_empty(),
        "{} of {} programs differ:\n\n{}",
        failures.len(),
        stems.len(),
        failures.join("\n\n")
    );
}
