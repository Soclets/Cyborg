//! Verus proves the compiler's checks (rule 13, D66, D67). Verus is a tool for working on the compiler, not for using
//! it: nothing here runs when someone builds a program.
#![cfg(target_os = "macos")]
#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![warn(clippy::pedantic)]
#![allow(clippy::expect_used, clippy::panic)]

use std::path::{Path, PathBuf};
use std::process::Command;

/// Verus, from `OKAY_VERUS` or where it's installed on the owner's Mac.
fn verus() -> PathBuf {
    if let Some(path) = std::env::var_os("OKAY_VERUS") {
        return PathBuf::from(path);
    }
    let home = std::env::var_os("HOME").expect("HOME is set");
    let path = Path::new(&home).join(".local/verus/verus-arm64-macos/verus");
    assert!(path.exists(), "Verus isn't at {} (OKAY_VERUS can say where it is)", path.display());
    path
}

/// Each file of proven checks.
const CHECKS: [&str; 2] = ["src/flow/checks.rs", "src/ranges/checks.rs"];

fn source(file: &str) -> String {
    std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(file)).expect("reading the checks")
}

// trace: D66, D67
#[test]
fn decision_66_verus_proves_the_checks() {
    for file in CHECKS {
        let output = Command::new(verus())
            .current_dir(env!("CARGO_MANIFEST_DIR"))
            .args([file, "--crate-type=lib", "--", "-Zcrate-attr=feature(proc_macro_hygiene)"])
            .output()
            .expect("running Verus");
        let text = format!("{}{}", String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr));
        assert!(output.status.success(), "{file}: {text}");
        let result = text.lines().find(|line| line.starts_with("verification results::")).expect("Verus's result line");
        assert!(result.ends_with(" 0 errors"), "{file}: {result}");
        let verified: usize = result
            .trim_start_matches("verification results:: ")
            .split_whitespace()
            .next()
            .and_then(|count| count.parse().ok())
            .expect("a count");
        let checks = source(file).matches("pub(crate) fn ").count();
        assert!(verified >= checks, "{file}: {verified} verified, for {checks} checks: {result}");
    }
}

// trace: D66
#[test]
fn decision_66_every_check_has_a_specification() {
    for file in CHECKS {
        let source = source(file);
        let lines: Vec<&str> = source.lines().collect();
        for (index, line) in lines.iter().enumerate() {
            if line.starts_with("pub(crate) fn ") {
                let attributes = lines[..index].iter().rev().take_while(|above| !above.starts_with("///"));
                assert!(
                    attributes.clone().any(|above| above.contains("verus_spec")),
                    "this check isn't proven: {line}"
                );
            }
        }
    }
}
