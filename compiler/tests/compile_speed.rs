//! Compiling fast (D168, rule 10): a program is built in units, on several cores, and each unit's object is cached by
//! its IR, so only what changed is built again. These are the compiler's own tests, since the oracle has no cache.

#![forbid(unsafe_code)]

use std::path::{Path, PathBuf};
use std::process::Command;

const MANIFEST: &str =
    "name speed\nver 2026y-11m-10d same\nokay 2026y-11m-10d\n\nneed std 2026y-11m-10d\nneed os 2026y-11m-10d\n";

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
    // The runtime and each of the three functions.
    let first = cached(&cache);
    assert_eq!(first, 4);
    assert_eq!(build(&folder, &cache, &flags), "4 5\n");
    assert_eq!(cached(&cache), first, "a build with nothing changed made new objects");
    // Changing one function's text builds only that function again.
    std::fs::write(folder.join("main.cy"), program(5)).expect("written");
    assert_eq!(build(&folder, &cache, &flags), "4 9\n");
    assert_eq!(cached(&cache), first + 1, "changing one function rebuilt more than it");
}

#[test]
fn d168_a_final_build_is_cached_whole() {
    let (folder, cache) = project("final");
    std::fs::write(folder.join("main.cy"), program(1)).expect("written");
    assert_eq!(build(&folder, &cache, &["--FinalPerf", "--max_cores=1"]), "4 5\n");
    // The runtime, and the program's functions together, so LLVM can inline across them.
    assert_eq!(cached(&cache), 2);
    assert_eq!(build(&folder, &cache, &["--FinalPerf", "--max_cores=1"]), "4 5\n");
    assert_eq!(cached(&cache), 2);
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
