//! What each build mode leaves in the program (standard 10.3, 10.4). The oracle's tests run only `Working`, so these
//! are the compiler's own, from two bugs benchmarks found: `FinalPerf` and `FinalShit` kept the overflow check on `+`,
//! which also stopped LLVM folding loops, and `FinalSmall` didn't build, since LLVM 23 dropped the `Oz` pipeline.

#![forbid(unsafe_code)]

use std::path::{Path, PathBuf};
use std::process::Command;

use okay::target::{Mode, Optimization, Target};

const MANIFEST: &str = "name modes\nver 2026y-11m-10d same\ndialect original 2026y-11m-10d\n\nneed std 2026y-11m-10d\nneed os 2026y-11m-10d\nneed concur 2026y-11m-10d\n";

/// Sums 1 to 100 million in an `i64`, which never overflows, so a mode with no checks can fold the loop to its answer.
const SUM: &str = "import std;
import os;
use std::io::fmt_print as fmt_print;

fn.main [none] -> void:
    let mut $out: os::stdout.buffered = os::stdout.buffered(room: 1024 bytes);
    let mut $sum: i64 = 0;
    let mut $i: i64 = 1;
    while $i at_most 100_000_000:
        change $sum = $sum + $i;
        change $i = $i + 1;
    end while
    fmt_print(Lend.mut $out, \"{$sum}{newln}\");
    os::done(Give $out);
    return void;
end fn.main
";

/// The same sum in an `i32`, which overflows while running.
const OVERFLOWING: &str = "import std;
import os;
use std::io::fmt_print as fmt_print;

fn.main [none] -> void:
    let mut $out: os::stdout.buffered = os::stdout.buffered(room: 1024 bytes);
    let mut $sum: i32 = 0;
    let mut $i: i32 = 1;
    while $i at_most 100_000:
        change $sum = $sum + $i;
        change $i = $i + 1;
    end while
    fmt_print(Lend.mut $out, \"{$sum}{newln}\");
    os::done(Give $out);
    return void;
end fn.main
";

/// A folder of its own for one program and its manifest.
fn project(name: &str, program: &str) -> PathBuf {
    let folder = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = std::fs::create_dir_all(&folder);
    std::fs::write(folder.join("CyJet.siml"), MANIFEST).expect("the manifest is written");
    let file = folder.join("main.cy");
    std::fs::write(&file, program).expect("the program is written");
    file
}

fn ir(name: &str, program: &str, mode: Mode) -> String {
    let file = project(name, program);
    let checked = okay::build::checked(&file, Target::host(), mode).expect("the program is checked");
    okay::codegen::optimized_ir(&checked, Target::host(), mode).expect("LLVM builds it")
}

/// Builds and runs a program, giving its exit code and standard output.
fn run(name: &str, program: &str, flags: &[&str]) -> (Option<i32>, String) {
    let file = project(name, program);
    let mut arguments = vec!["comp", "--max_cores=1"];
    arguments.extend_from_slice(flags);
    arguments.push("main.cy");
    let folder = file.parent().expect("the program is in a folder");
    let built =
        Command::new(env!("CARGO_BIN_EXE_okay")).args(&arguments).current_dir(folder).output().expect("okay runs");
    assert!(built.status.success(), "okay {arguments:?} failed:\n{}", String::from_utf8_lossy(&built.stderr));
    let ran = Command::new(folder.join("main")).output().expect("the program runs");
    (ran.status.code(), String::from_utf8_lossy(&ran.stdout).into_owned())
}

#[test]
fn d14_final_perf_and_final_shit_have_no_overflow_check() {
    for (name, mode) in [("perf_ir", Mode::FinalPerf), ("shit_ir", Mode::FinalShit)] {
        let text = ir(name, SUM, mode);
        assert!(!text.contains("with.overflow"), "{} kept an overflow check:\n{text}", mode.name());
        // With nothing in the way, LLVM works out the whole sum while compiling.
        assert!(text.contains("5000000050000000"), "{} didn't fold the loop:\n{text}", mode.name());
    }
}

#[test]
fn d14_final_safe_keeps_the_overflow_check() {
    let text = ir("safe_ir", SUM, Mode::FinalSafe);
    assert!(text.contains("with.overflow"), "FinalSafe dropped its overflow check:\n{text}");
}

#[test]
fn d42_a_plain_plus_wraps_in_final_perf() {
    // 1 + 2 + … + 100000 is 5000050000, which wraps to 705082704 in an `i32`.
    assert_eq!(run("perf_wraps", OVERFLOWING, &["--FinalPerf"]), (Some(0), "705082704\n".to_string()));
}

#[test]
fn d42_a_plain_plus_still_traps_in_working() {
    let (code, _) = run("working_traps", OVERFLOWING, &["--Working", "--opt=O2"]);
    assert_eq!(code, Some(1));
}

#[test]
fn d288_final_small_and_size_levels_build() {
    for (name, flags) in [
        ("small", &["--FinalSmall"][..]),
        ("working_oz", &["--Working", "--opt=Oz"][..]),
        ("working_os", &["--Working", "--opt=Os"][..]),
    ] {
        assert_eq!(run(name, SUM, flags), (Some(0), "5000000050000000\n".to_string()), "{flags:?}");
    }
    let text = ir("small_ir", SUM, Mode::FinalSmall);
    assert!(text.contains("minsize"), "FinalSmall's functions aren't marked minsize:\n{text}");
    let text = ir("oz_ir", SUM, Mode::Working(Optimization::Oz));
    assert!(text.contains("minsize"), "`--opt=Oz` functions aren't marked minsize:\n{text}");
}

/// Structs whose layout is chosen by name: `.packed` has no padding, and `.compact` sorts its fields by alignment.
const LAYOUTS: &str = "\
import std;
import os;

struct.packed header:
    kind: u8;
    length: u32;
end struct.packed header

struct.compact sorted:
    flag: u8;
    when: u64;
    level: u32;
end struct.compact sorted

fn kind_of [$h: Lent header] -> u8:
    return $h.kind;
end fn kind_of

fn level_of [$s: Lent sorted] -> u32:
    return $s.level;
end fn level_of

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $h: header = header[kind = 1, length = 512];
    let $s: sorted = sorted[flag = 1, when = 2, level = 3];
    std::io::fmt_print(Lend.mut $out, \"{} {} {} {}{newln}\", kind_of(Lend $h), $h.length, level_of(Lend $s), $s.when);
    return void;
end fn.main
";

#[test]
fn d319_d388_layouts_are_what_the_struct_says() {
    // trace: D319, D388
    let text = ir("layouts_ir", LAYOUTS, Mode::Working(Optimization::O0));
    assert!(text.contains("<{ i8, i32 }>"), "a `.packed` struct has padding:\n{text}");
    assert!(text.contains("{ i64, i32, i8 }"), "a `.compact` struct isn't sorted by alignment:\n{text}");
    let (code, out) = run("layouts_run", LAYOUTS, &["--Working", "--opt=O0"]);
    assert_eq!((code, out.as_str()), (Some(0), "1 512 3 2\n"));
}

const DEFER_PASS_UP: &str = "import std;
import os;

fn half [$n: i64] -> i64 fails [odd]:
    if $n mod.trunc 2 == 1:
        fail odd;
    end if
    return $n div.trunc 2;
end fn half

fn both [$n: i64] -> i64 fails [odd]:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    defer std::io::fmt_print(Lend.mut $out, \"left both{newln}\");
    let $h: i64 = half($n)?;
    return $h;
end fn both

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $a: i64 = both(4) otherwise:
        case odd:
            instead -1;
    end otherwise
    let $b: i64 = both(3) otherwise:
        case odd:
            instead -1;
    end otherwise
    std::io::fmt_print(Lend.mut $out, \"{} {}{newln}\", $a, $b);
    return void;
end fn.main
";

#[test]
fn d325_leaving_through_a_question_mark_runs_what_was_deferred() {
    // trace: D325
    let (code, out) = run("defer_pass_up", DEFER_PASS_UP, &["--FinalSafe"]);
    assert_eq!((code, out.as_str()), (Some(0), "left both\nleft both\n2 -1\n"));
}

const ATOMICS: &str = "import std;
import os;
import @concur;

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $hits: @concur::atomic<u64> = @concur::atomic.new(0);
    $hits.%add(val: 1);
    $hits.%add.wrap.relaxed(val: 2);
    let $old: u64 = $hits.%swap(val: 7);
    let $owner: @concur::atomic<i32> = @concur::atomic.new(0);
    $owner.%compare_swap(expect: 0, val: 5) otherwise:
        case not_expected $f:
            std::io::fmt_print(Lend.mut $out, \"lost to {}{newln}\", $f.val);
    end otherwise
    std::io::fmt_print(Lend.mut $out, \"{} {} {}{newln}\", $old, $hits.%load.acquire(), $owner.%load());
    return void;
end fn.main
";

#[test]
fn d354_d360_atomics_are_the_cpus_own_instructions() {
    // trace: D354, D360
    let text = ir("atomics_ir", ATOMICS, Mode::Working(Optimization::O0));
    assert!(text.contains("atomicrmw add ptr") && text.contains("monotonic"), "a `.wrap.relaxed` add:\n{text}");
    assert!(text.contains("atomicrmw xchg ptr"), "a swap:\n{text}");
    assert!(text.contains("cmpxchg ptr"), "a checked add or a compare_swap:\n{text}");
    assert!(text.contains("load atomic i64") && text.contains("acquire"), "an acquiring load:\n{text}");
    let (code, out) = run("atomics_run", ATOMICS, &["--Working", "--opt=O0"]);
    assert_eq!((code, out.as_str()), (Some(0), "3 7 5\n"));
}
