//! Compiling fast (D168, rule 10): a program is built in units, on several cores, and each unit's object is cached by
//! its IR, so only what changed is built again. These are the compiler's own tests, since the oracle has no cache.

#![forbid(unsafe_code)]

use std::path::{Path, PathBuf};
use std::process::Command;

const MANIFEST: &str = "name speed\nver 2026y-11m-10d same\ndialect original 2026y-11m-10d\n\nneed std 2026y-11m-10d\nneed os 2026y-11m-10d\n";

fn program(added: i32) -> String {
    format!(
        "import std;
import os;
use std::io::fmt_print as fmt_print;

fn double [$n: i32] -> i32:
    return $n x 2;
end fn double

fn bump [$n: i32] -> i32:
    return $n + {added};
end fn bump

fn.main [none] -> void:
    let mut $out: os::stdout.buffered = os::stdout.buffered(size: 1024 bytes);
    let $four: i32 = double(2);
    let $more: i32 = bump($four);
    fmt_print(Lend.mut $out, \"{{$four}} {{$more}}{{newln}}\");
    os::done(Give $out);
    return void;
end fn.main
"
    )
}

/// A fresh folder with a fresh cache of its own.
fn project(name: &str) -> (PathBuf, PathBuf) {
    let folder = Path::new(env!("CARGO_TARGET_TMPDIR")).join("speed").join(name);
    let _ = std::fs::remove_dir_all(&folder);
    std::fs::create_dir_all(&folder).expect("the folder is made");
    std::fs::write(folder.join("CyJet.siml"), MANIFEST).expect("the manifest is written");
    let cache = folder.join("cache");
    (folder, cache)
}

/// Builds `main.cy` and runs it, giving what it printed.
fn build(folder: &Path, cache: &Path, flags: &[&str]) -> String {
    let built = Command::new(env!("CARGO_BIN_EXE_okay"))
        .arg("comp")
        .args(flags)
        .arg("main.cy")
        .env("OKAY_CACHE", cache)
        .current_dir(folder)
        .output()
        .expect("okay runs");
    assert!(built.status.success(), "okay {flags:?} failed:\n{}", String::from_utf8_lossy(&built.stderr));
    let ran = Command::new(folder.join("main")).output().expect("the program runs");
    assert!(ran.status.success());
    String::from_utf8_lossy(&ran.stdout).into_owned()
}

fn cached(cache: &Path) -> usize {
    std::fs::read_dir(cache.join("objects")).map_or(0, Iterator::count)
}

#[test]
fn d168_only_what_changed_is_built_again() {
    let (folder, cache) = project("incremental");
    let flags = ["--Working", "--opt=O1", "--max_cores=1"];
    std::fs::write(folder.join("main.cy"), program(1)).expect("written");
    assert_eq!(build(&folder, &cache, &flags), "4 5\n");
    // The runtime, each of the three functions, and the crash messages.
    let first = cached(&cache);
    assert_eq!(first, 5);
    assert_eq!(build(&folder, &cache, &flags), "4 5\n");
    assert_eq!(cached(&cache), first, "a build with nothing changed made new objects");
    // Changing one function's text builds only that function again, and the messages, whose text quotes its line.
    std::fs::write(folder.join("main.cy"), program(5)).expect("written");
    assert_eq!(build(&folder, &cache, &flags), "4 9\n");
    assert_eq!(cached(&cache), first + 2, "changing one function rebuilt more than it");
}

#[test]
fn d168_a_final_build_is_cached_whole() {
    let (folder, cache) = project("final");
    std::fs::write(folder.join("main.cy"), program(1)).expect("written");
    assert_eq!(build(&folder, &cache, &["--FinalPerf", "--max_cores=1"]), "4 5\n");
    // The runtime, the program's functions together, so LLVM can inline across them, and the crash messages.
    assert_eq!(cached(&cache), 3);
    assert_eq!(build(&folder, &cache, &["--FinalPerf", "--max_cores=1"]), "4 5\n");
    assert_eq!(cached(&cache), 3);
}

#[test]
fn d224_cores_change_nothing_but_speed() {
    let (folder, cache) = project("cores");
    std::fs::write(folder.join("main.cy"), program(1)).expect("written");
    let one = build(&folder, &cache, &["--Working", "--opt=O0", "--max_cores=1"]);
    let one_program = std::fs::read(folder.join("main")).expect("built");
    let _ = std::fs::remove_dir_all(&cache);
    let three = build(&folder, &cache, &["--Working", "--opt=O0", "--max_cores=3"]);
    assert_eq!(one, three);
    assert_eq!(one_program, std::fs::read(folder.join("main")).expect("built"), "the programs differ");
}

#[test]
fn d224_max_cores_takes_a_number_max_or_min() {
    let (folder, cache) = project("flag");
    std::fs::write(folder.join("main.cy"), program(1)).expect("written");
    assert_eq!(build(&folder, &cache, &["--Working", "--opt=O0", "--max_cores=min"]), "4 5\n");
    for wrong in ["--max_cores=0", "--max_cores=lots"] {
        let refused = Command::new(env!("CARGO_BIN_EXE_okay"))
            .args(["comp", "--Working", "--opt=O0", wrong, "main.cy"])
            .env("OKAY_CACHE", &cache)
            .current_dir(&folder)
            .output()
            .expect("okay runs");
        assert_eq!(refused.status.code(), Some(2), "{wrong} wasn't refused");
    }
}

fn checked(folder: &Path, cache: &Path) -> (Result<okay::checked::Program, String>, okay::check::incremental::Stats) {
    okay::build::checked_incrementally(
        &folder.join("main.cy"),
        okay::target::Target::host(),
        okay::target::Mode::Working(okay::target::Optimization::O0),
        Some(cache),
    )
}

#[test]
fn d168_checking_again_checks_only_what_changed() {
    let (folder, cache) = project("checking");
    std::fs::write(folder.join("main.cy"), program(1)).expect("written");
    let (first, stats) = checked(&folder, &cache);
    assert!(first.is_ok());
    assert_eq!((stats.checked, stats.reused), (3, 0));
    let (_, stats) = checked(&folder, &cache);
    assert_eq!((stats.checked, stats.reused), (0, 3), "nothing changed, and something was checked again");
    // A body changed, and its interface didn't, so only it is checked again.
    std::fs::write(folder.join("main.cy"), program(5)).expect("written");
    let (_, stats) = checked(&folder, &cache);
    assert_eq!((stats.checked, stats.reused), (1, 2));
}

#[test]
fn d168_a_changed_interface_checks_its_callers_again() {
    let (folder, cache) = project("interface");
    std::fs::write(folder.join("main.cy"), program(1)).expect("written");
    assert!(checked(&folder, &cache).0.is_ok());
    // `double` now gives back an `i64`, which `main` stores in an `i32`, so `main` has to be checked again.
    let changed = program(1)
        .replace("fn double [$n: i32] -> i32:\n    return $n x 2;", "fn double [$n: i64] -> i64:\n    return $n x 2;");
    std::fs::write(folder.join("main.cy"), changed).expect("written");
    let (result, stats) = checked(&folder, &cache);
    assert!(result.is_err(), "main kept its old check, though what it calls changed");
    assert_eq!((stats.checked, stats.reused), (2, 1));
}

/// A program whose check fails while running, in a function below `main`'s callers.
const FAILING: &str = "import std;
import os;

fn small [$n: i32] -> i32:
    let $x: i32[min: 0, max: 10] = $n;
    return $x;
end fn small

fn.main [none] -> void:
    let $y: i32 = small(50);
    return void;
end fn.main
";

fn crash(folder: &Path, cache: &Path) -> String {
    let built = Command::new(env!("CARGO_BIN_EXE_okay"))
        .args(["comp", "--Working", "--opt=O0", "--max_cores=1", "main.cy"])
        .env("OKAY_CACHE", cache)
        .current_dir(folder)
        .output()
        .expect("okay runs");
    assert!(built.status.success(), "{}", String::from_utf8_lossy(&built.stderr));
    let ran = Command::new(folder.join("main")).output().expect("the program runs");
    String::from_utf8_lossy(&ran.stderr).into_owned()
}

#[test]
fn d168_a_kept_function_that_moved_shows_its_new_lines() {
    let (folder, cache) = project("moved");
    std::fs::write(folder.join("main.cy"), FAILING).expect("written");
    let before = crash(&folder, &cache);
    assert!(before.contains("line 5"), "{before}");
    std::fs::write(folder.join("main.cy"), FAILING.replace("import os;\n", "import os;\n\n// moved down\n"))
        .expect("written");
    let moved = crash(&folder, &cache);
    let fresh = crash(&folder, &folder.join("fresh cache"));
    assert!(moved.contains("line 7"), "{moved}");
    assert_eq!(moved, fresh, "a kept function's message differs from a fresh build's");
}

#[test]
fn d168_adding_or_moving_a_function_checks_only_what_names_it() {
    let (folder, cache) = project("adding");
    std::fs::write(folder.join("main.cy"), program(1)).expect("written");
    assert!(checked(&folder, &cache).0.is_ok());
    // A new function above the others moves every function's number, and nothing else names it.
    let added = program(1).replacen(
        "fn double",
        "fn triple [$n: i32] -> i32:\n    return $n x 3;\nend fn triple\n\nfn double",
        1,
    );
    std::fs::write(folder.join("main.cy"), &added).expect("written");
    let (result, stats) = checked(&folder, &cache);
    assert!(result.is_ok());
    assert_eq!((stats.checked, stats.reused), (1, 3), "adding a function checked more than it");
    // Moving `bump` above `triple` changes numbers again, and checks nothing.
    let bump = "fn bump [$n: i32] -> i32:\n    return $n + 1;\nend fn bump\n\n";
    let moved = added.replace(bump, "").replacen("fn triple", &format!("{bump}fn triple"), 1);
    std::fs::write(folder.join("main.cy"), &moved).expect("written");
    let (result, stats) = checked(&folder, &cache);
    assert!(result.is_ok(), "{result:?}");
    assert_eq!((stats.checked, stats.reused), (0, 4), "moving a function checked something again");
    // The program still runs right, with every call going to the function it named.
    assert_eq!(build(&folder, &cache, &["--Working", "--opt=O0", "--max_cores=1"]), "4 5\n");
}

#[test]
fn d168_adding_a_struct_checks_only_what_reaches_it() {
    let (folder, cache) = project("struct");
    std::fs::write(folder.join("main.cy"), program(1)).expect("written");
    assert!(checked(&folder, &cache).0.is_ok());
    let added =
        program(1).replacen("fn double", "struct reading:\n    celsius: i32;\nend struct reading\n\nfn double", 1);
    std::fs::write(folder.join("main.cy"), &added).expect("written");
    let (result, stats) = checked(&folder, &cache);
    assert!(result.is_ok(), "{result:?}");
    assert_eq!((stats.checked, stats.reused), (0, 3), "a struct nothing uses checked something again");
    // `bump` now uses one, so it's checked again, and nothing else, since its interface is the same.
    let used = added.replace(
        "fn bump [$n: i32] -> i32:\n    return $n + 1;",
        "fn bump [$n: i32] -> i32:\n    let $r: reading = reading[celsius = $n];\n    return $r.celsius + 1;",
    );
    std::fs::write(folder.join("main.cy"), &used).expect("written");
    let (result, stats) = checked(&folder, &cache);
    assert!(result.is_ok(), "{result:?}");
    assert_eq!((stats.checked, stats.reused), (1, 2));
}

#[test]
fn d168_lines_moving_rebuilds_only_the_messages() {
    let (folder, cache) = project("lines");
    std::fs::write(folder.join("main.cy"), FAILING).expect("written");
    let before = crash(&folder, &cache);
    let first = cached(&cache);
    // Two lines above everything move every message's line, and nothing else.
    std::fs::write(folder.join("main.cy"), FAILING.replace("import os;\n", "import os;\n\n// moved down\n"))
        .expect("written");
    let moved = crash(&folder, &cache);
    assert_ne!(before, moved);
    assert!(moved.contains("line 7"), "{moved}");
    assert_eq!(cached(&cache), first + 1, "moving lines rebuilt more than the messages");
}
