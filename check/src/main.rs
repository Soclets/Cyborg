//! `check`: Cyborg's random program generator and the live comparison of the oracle and
//! `OKAY\n`, written from `standard/`, never from either's source. (D68, D313, rule 3)
//!
//! ```text
//! check generate <seed> <count> <folder>
//! check compare <oracle> <okay> <program.cy or folder>...
//! check fuzz <oracle> <okay> <seed> <count> <folder>
//! check coverage
//! ```
#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![warn(clippy::pedantic)]

mod compare;
mod coverage;
mod generate;
mod random;
mod shrink;
mod tree;

use std::path::{Path, PathBuf};
use std::process::ExitCode;

const MANIFEST: &str = "name generated\nver 2026y-11m-12d same\ndialect original 2026y-11m-12d\n\nneed std 2026y-11m-12d\nneed os 2026y-11m-12d\nneed concur 2026y-11m-12d\n";

const USAGE: &str = "usage:
  check generate <seed> <count> <folder>
  check compare <oracle> <okay> <program.cy or folder>...
  check fuzz <oracle> <okay> <seed> <count> <folder>
  check coverage";

fn main() -> ExitCode {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let words: Vec<&str> = arguments.iter().map(String::as_str).collect();
    let result =
        match words.as_slice() {
            ["generate", seed, count, folder] => generate_programs(seed, count, Path::new(folder)),
            ["compare", oracle, okay, programs @ ..] if !programs.is_empty() => Ok(
                compare_programs(Path::new(oracle), Path::new(okay), programs),
            ),
            ["fuzz", oracle, okay, seed, count, folder] => fuzz(
                Path::new(oracle),
                Path::new(okay),
                seed,
                count,
                Path::new(folder),
            ),
            ["coverage"] => {
                println!("Not generated yet:");
                for line in coverage::NOT_GENERATED {
                    println!("  {line}");
                }
                Ok(true)
            }
            _ => Err(USAGE.to_string()),
        };
    match result {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::from(1),
        Err(problem) => {
            eprintln!("{problem}");
            ExitCode::from(2)
        }
    }
}

fn number(text: &str, what: &str) -> Result<u64, String> {
    text.parse()
        .map_err(|_| format!("check: `{text}` isn't a {what}\n{USAGE}"))
}

/// A folder of generated programs, sharing one manifest.
fn prepare(folder: &Path) -> Result<(), String> {
    std::fs::create_dir_all(folder)
        .and_then(|()| std::fs::write(folder.join("CyJet.siml"), MANIFEST))
        .map_err(|problem| format!("check: couldn't write to {}: {problem}", folder.display()))
}

fn generate_programs(seed: &str, count: &str, folder: &Path) -> Result<bool, String> {
    let seed = number(seed, "seed")?;
    let count = number(count, "count")?;
    prepare(folder)?;
    for offset in 0..count {
        let program = generate::program(seed.wrapping_add(offset));
        let path = folder.join(format!("generated_{}.cy", seed.wrapping_add(offset)));
        std::fs::write(&path, tree::print(&program))
            .map_err(|problem| format!("check: couldn't write {}: {problem}", path.display()))?;
    }
    println!("wrote {count} programs to {}", folder.display());
    Ok(true)
}

/// Every `.cy` file named, or in a folder named, with its subfolders.
fn programs_in(paths: &[&str]) -> Vec<PathBuf> {
    let mut found = Vec::new();
    for path in paths {
        let path = Path::new(path);
        if path.is_dir() {
            let mut entries: Vec<PathBuf> = std::fs::read_dir(path)
                .map(|entries| {
                    entries
                        .filter_map(Result::ok)
                        .map(|entry| entry.path())
                        .collect()
                })
                .unwrap_or_default();
            entries.sort();
            for entry in entries {
                if entry.is_dir() {
                    found.extend(programs_in(&[&entry.to_string_lossy()]));
                } else if entry.extension().is_some_and(|extension| extension == "cy") {
                    found.push(entry);
                }
            }
        } else {
            found.push(path.to_path_buf());
        }
    }
    found
}

/// Tallies, and prints every difference.
#[derive(Default)]
struct Tally {
    same: usize,
    different: usize,
    undecided: usize,
    unbuilt: usize,
    close: usize,
}

impl Tally {
    fn add(&mut self, program: &Path, expected: &compare::Outcome, verdict: &compare::Verdict) {
        match verdict {
            compare::Verdict::Same => self.same += 1,
            compare::Verdict::OutOfScope("NotBuiltYet") => self.unbuilt += 1,
            compare::Verdict::OutOfScope(_) => self.undecided += 1,
            compare::Verdict::Close => self.close += 1,
            compare::Verdict::Different(modes) => {
                self.different += 1;
                println!("DIFFERENT {}", program.display());
                // Modes that gave the same outcome are reported together.
                let mut groups: Vec<(Vec<String>, &compare::Outcome)> = Vec::new();
                for (mode, found) in modes {
                    match groups.iter_mut().find(|(_, outcome)| *outcome == found) {
                        Some((names, _)) => names.push(mode.name()),
                        None => groups.push((vec![mode.name()], found)),
                    }
                }
                for (names, found) in groups {
                    println!("  {}:", names.join(", "));
                    if found.out != expected.out {
                        println!(
                            "    stdout: oracle {:?}, okay {:?}",
                            expected.out, found.out
                        );
                    }
                    if found.error != expected.error {
                        println!(
                            "    stderr: oracle {:?}, okay {:?}",
                            first_line(&expected.error),
                            first_line(&found.error)
                        );
                    }
                    if found.code != expected.code {
                        println!(
                            "    exit: oracle {:?}, okay {:?}",
                            expected.code, found.code
                        );
                    }
                }
            }
        }
    }

    fn report(&self) -> bool {
        println!(
            "{} same, {} different, {} out of scope as NotDesignedYet, {} as NotBuiltYet, {} `.fast` left to judge by hand",
            self.same, self.different, self.undecided, self.unbuilt, self.close
        );
        self.different == 0
    }
}

fn first_line(text: &str) -> &str {
    text.lines().next().unwrap_or("")
}

fn compare_programs(oracle: &Path, okay: &Path, programs: &[&str]) -> bool {
    let modes = compare::every_mode();
    let mut tally = Tally::default();
    for program in programs_in(programs) {
        let (expected, verdict) = compare::compare(oracle, okay, &modes, &program);
        tally.add(&program, &expected, &verdict);
    }
    tally.report()
}

/// Generates programs, compares each, and shrinks every difference, keeping the smallest
/// program as `<seed>.smallest.cy` beside it.
fn fuzz(
    oracle: &Path,
    okay: &Path,
    seed: &str,
    count: &str,
    folder: &Path,
) -> Result<bool, String> {
    let seed = number(seed, "seed")?;
    let count = number(count, "count")?;
    prepare(folder)?;
    let modes = compare::every_mode();
    let mut tally = Tally::default();
    for offset in 0..count {
        let this_seed = seed.wrapping_add(offset);
        let program = generate::program(this_seed);
        let path = folder.join(format!("generated_{this_seed}.cy"));
        std::fs::write(&path, tree::print(&program))
            .map_err(|problem| format!("check: couldn't write {}: {problem}", path.display()))?;
        let (expected, verdict) = compare::compare(oracle, okay, &modes, &path);
        tally.add(&path, &expected, &verdict);
        if let compare::Verdict::Different(modes) = &verdict {
            if let Some((mode, _)) = modes.first() {
                let smallest = folder.join(format!("generated_{this_seed}.smallest.cy"));
                shrink::shrink(oracle, okay, mode, &smallest, &program);
                println!("  shrunk to {}", smallest.display());
            }
        }
    }
    Ok(tally.report())
}
