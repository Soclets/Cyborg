//! Checks `// trace: D13` tags: a tag naming no decision fails, and the decisions nothing traces
//! yet are listed. Run it from the repository's top with `cargo run --manifest-path tools/Cargo.toml`.
#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![warn(clippy::pedantic)]

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

/// Every decision number `DECISIONS.md` declares, as in `**D13. Labels can have narrowings.**`.
fn declared_decisions(text: &str) -> BTreeSet<u32> {
    text.lines()
        .filter_map(|line| line.strip_prefix("**D"))
        .filter_map(|rest| rest.split('.').next())
        .filter_map(|number| number.parse().ok())
        .collect()
}

fn files_under(folder: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(folder) else {
        return;
    };
    for entry in entries.filter_map(Result::ok) {
        let path = entry.path();
        if path.is_dir() {
            if path.file_name().is_some_and(|name| name != "target") {
                files_under(&path, out);
            }
        } else if path
            .extension()
            .is_some_and(|extension| extension == "rs" || extension == "cy")
        {
            out.push(path);
        }
    }
}

/// The numbers in a line's trace tag, as in `// trace: D2, D5`. `/// trace:` counts too.
fn tags_in(line: &str) -> Vec<String> {
    let Some((_, after)) = line.split_once("trace: D") else {
        return Vec::new();
    };
    let is_comment = line.trim_start().starts_with("//");
    if !is_comment {
        return Vec::new();
    }
    format!("D{after}")
        .split(',')
        .map(|tag| tag.trim().trim_start_matches('D').to_string())
        .collect()
}

fn main() -> ExitCode {
    let Ok(decisions) = std::fs::read_to_string("DECISIONS.md") else {
        eprintln!("trace: run this from the repository's top, where DECISIONS.md is");
        return ExitCode::from(2);
    };
    let declared = declared_decisions(&decisions);
    let mut files = Vec::new();
    for folder in ["oracle/src", "tests", "compiler"] {
        files_under(Path::new(folder), &mut files);
    }
    files.sort();
    let mut traced = BTreeSet::new();
    let mut bad = 0;
    for file in &files {
        let Ok(text) = std::fs::read_to_string(file) else {
            continue;
        };
        for (index, line) in text.lines().enumerate() {
            for tag in tags_in(line) {
                match tag.parse::<u32>() {
                    Ok(number) if declared.contains(&number) => {
                        traced.insert(number);
                    }
                    _ => {
                        eprintln!(
                            "{}:{}: `D{tag}` names no decision",
                            file.display(),
                            index + 1
                        );
                        bad += 1;
                    }
                }
            }
        }
    }
    let untraced: Vec<String> = declared
        .difference(&traced)
        .map(|number| format!("D{number}"))
        .collect();
    println!(
        "{} of {} decisions traced, {} files read",
        traced.len(),
        declared.len(),
        files.len()
    );
    println!("nothing traces yet: {}", untraced.join(" "));
    if bad > 0 {
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}
