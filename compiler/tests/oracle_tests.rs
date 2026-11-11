//! Runs the oracle's tests against `okay` (rule 5): each program's standard output, its error and its exit code
//! must match byte for byte.
//!
//! `claimed.txt` lists the tests this compiler claims to pass, and every one of them must. With `OKAY_ALL_TESTS=1`,
//! every test runs instead, and the ones that pass are written to `passing.txt`, so progress can be seen.

#![forbid(unsafe_code)]

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Mutex;

/// One test: the folder it runs in and the program's file name there.
#[derive(Clone)]
struct Test {
    name: String,
    folder: PathBuf,
    program: String,
}

fn tests_folder() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("tests")
}

fn all_tests() -> Vec<Test> {
    let folder = tests_folder();
    let mut tests = Vec::new();
    let Ok(entries) = std::fs::read_dir(&folder) else {
        return tests;
    };
    let mut entries: Vec<_> = entries.filter_map(Result::ok).map(|entry| entry.path()).collect();
    entries.sort();
    for path in entries {
        let Some(name) = path.file_name().map(|name| name.to_string_lossy().into_owned()) else {
            continue;
        };
        if path.is_dir() && path.join("main.cy").exists() {
            tests.push(Test { name, folder: path.clone(), program: "main.cy".to_string() });
        } else if let Some(stem) = name.strip_suffix(".cy") {
            tests.push(Test { name: stem.to_string(), folder: folder.clone(), program: name.clone() });
        }
    }
    tests
}

/// Runs one test, giving what differs if it fails.
fn run(test: &Test) -> Result<(), String> {
    let stem = test.program.trim_end_matches(".cy");
    let expected = |extension: &str| std::fs::read(test.folder.join(format!("{stem}.{extension}"))).unwrap_or_default();
    let expected_out = expected("out");
    let expected_error = expected("err");
    let expected_exit = match std::fs::read_to_string(test.folder.join(format!("{stem}.exit"))) {
        Ok(code) => code.trim().parse::<i32>().unwrap_or(-1),
        Err(_) if expected_error.is_empty() => 0,
        Err(_) => 1,
    };
    let output = Command::new(env!("CARGO_BIN_EXE_okay"))
        .args(["run", "--Working", &test.program])
        .current_dir(&test.folder)
        .output()
        .map_err(|error| format!("couldn't run okay: {error}"))?;
    let mut problems = Vec::new();
    if output.stdout != expected_out {
        problems.push(format!(
            "stdout differs:\n--- expected\n{}\n--- got\n{}",
            String::from_utf8_lossy(&expected_out),
            String::from_utf8_lossy(&output.stdout)
        ));
    }
    if output.stderr != expected_error {
        problems.push(format!(
            "stderr differs:\n--- expected\n{}\n--- got\n{}",
            String::from_utf8_lossy(&expected_error),
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    if output.status.code() != Some(expected_exit) {
        problems.push(format!("exit code: expected {expected_exit}, got {:?}", output.status.code()));
    }
    if problems.is_empty() { Ok(()) } else { Err(problems.join("\n")) }
}

/// Runs tests on three threads, the compiler session's share of the cores (rule 9).
fn run_all(tests: &[Test]) -> Vec<(String, Result<(), String>)> {
    let queue = Mutex::new(tests.iter());
    let results = Mutex::new(Vec::new());
    std::thread::scope(|scope| {
        for _ in 0..3 {
            scope.spawn(|| {
                loop {
                    let next = queue.lock().ok().and_then(|mut queue| queue.next().cloned());
                    let Some(test) = next else { break };
                    let result = run(&test);
                    if let Ok(mut results) = results.lock() {
                        results.push((test.name.clone(), result));
                    }
                }
            });
        }
    });
    let mut results = results.into_inner().unwrap_or_default();
    results.sort_by(|a, b| a.0.cmp(&b.0));
    results
}

#[test]
fn oracle_tests() {
    let all = all_tests();
    let here = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests");
    if std::env::var("OKAY_ALL_TESTS").is_ok() {
        let results = run_all(&all);
        let passing: Vec<&str> =
            results.iter().filter(|(_, result)| result.is_ok()).map(|(name, _)| name.as_str()).collect();
        let _ = std::fs::write(here.join("passing.txt"), passing.join("\n") + "\n");
        println!("{} of {} tests pass", passing.len(), results.len());
        return;
    }
    let claimed = std::fs::read_to_string(here.join("claimed.txt")).unwrap_or_default();
    let claimed: Vec<&str> =
        claimed.lines().map(str::trim).filter(|line| !line.is_empty() && !line.starts_with('#')).collect();
    let chosen: Vec<Test> = all.into_iter().filter(|test| claimed.contains(&test.name.as_str())).collect();
    assert_eq!(chosen.len(), claimed.len(), "claimed.txt names a test that doesn't exist");
    let failures: Vec<String> = run_all(&chosen)
        .into_iter()
        .filter_map(|(name, result)| result.err().map(|problem| format!("{name}:\n{problem}")))
        .collect();
    assert!(failures.is_empty(), "{} claimed tests fail:\n\n{}", failures.len(), failures.join("\n\n"));
}
