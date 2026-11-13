//! Dropping checks that can't fail (D312): outside `Working`, a check its ranges prove can never fire is gone from the
//! checked program, and one they can't prove stays. `Working` keeps every check (D280).

#![forbid(unsafe_code)]

use std::path::{Path, PathBuf};

use okay::target::{Mode, Optimization, Target};

const MANIFEST: &str = "name ranges\nver 2026y-11m-10d same\ndialect original 2026y-11m-10d\n\nneed std 2026y-11m-10d\nneed os 2026y-11m-10d\n";

const PROGRAM: &str = "import std;
import os;

fn bounded_sum [$a: i32[min: 0, max: 100], $b: i32[min: 0, max: 100]] -> i32:
    return $a + $b;
end fn bounded_sum

fn open_sum [$a: i32, $b: i32] -> i32:
    return $a + $b;
end fn open_sum

fn widened [$a: i32[min: 0, max: 10]] -> i32[min: 0, max: 20]:
    return $a + $a;
end fn widened

fn narrowed [$a: i32[min: 0, max: 10]] -> i32[min: 0, max: 5]:
    return $a;
end fn narrowed

fn halved [$a: i32] -> i32:
    return $a div.trunc 2;
end fn halved

fn divided [$a: i32, $b: i32] -> i32:
    return $a div.trunc $b;
end fn divided

fn second [none] -> i32:
    let $xs: std::coll::array<i32>[len: 3 items] = [1, 2, 3];
    return $xs[pos: 2];
end fn second

fn.main [none] -> void:
    let $a: i32 = bounded_sum(1, 2) + open_sum(3, 4) + widened(5) + narrowed(3) + halved(8) + divided(9, 3) + second();
    return void;
end fn.main
";

fn program(mode: Mode) -> okay::checked::Program {
    // Each test thread writes its own copy, so they don't read each other's half-written files.
    let thread =
        format!("{:?}", std::thread::current().id()).replace(|character: char| !character.is_alphanumeric(), "");
    let folder = Path::new(env!("CARGO_TARGET_TMPDIR")).join("dropped").join(thread);
    let _ = std::fs::create_dir_all(&folder);
    std::fs::write(folder.join("CyJet.siml"), MANIFEST).expect("written");
    let file: PathBuf = folder.join("main.cy");
    std::fs::write(&file, PROGRAM).expect("written");
    okay::build::checked(&file, Target::host(), mode).expect("checked")
}

/// A function's checked body, as text.
fn body(program: &okay::checked::Program, name: &str) -> String {
    let function = program.functions.iter().find(|function| function.symbol == format!("cyborg.{name}")).expect(name);
    format!("{:?}", function.body)
}

#[test]
fn d312_a_sum_of_bounded_values_loses_its_overflow_check() {
    let safe = program(Mode::FinalSafe);
    assert!(!body(&safe, "bounded_sum").contains("failure: Some"), "{}", body(&safe, "bounded_sum"));
    assert!(body(&safe, "open_sum").contains("failure: Some"), "an unprovable overflow check was dropped");
}

#[test]
fn d312_a_bound_that_always_holds_loses_its_check() {
    let safe = program(Mode::FinalSafe);
    assert!(!body(&safe, "widened").contains("Fit {"), "{}", body(&safe, "widened"));
    assert!(body(&safe, "narrowed").contains("Fit {"), "an unprovable bound check was dropped");
}

#[test]
fn d312_a_divisor_that_isnt_0_and_a_position_inside_lose_their_checks() {
    let safe = program(Mode::FinalSafe);
    assert!(!body(&safe, "halved").contains("failure: Some"), "{}", body(&safe, "halved"));
    assert!(body(&safe, "divided").contains("failure: Some"), "an unprovable division check was dropped");
    assert!(!body(&safe, "second").contains("check: Some"), "{}", body(&safe, "second"));
}

#[test]
fn d280_working_keeps_every_check() {
    let working = program(Mode::Working(Optimization::O0));
    assert!(body(&working, "bounded_sum").contains("failure: Some"));
    assert!(body(&working, "widened").contains("Fit {"));
    assert!(body(&working, "halved").contains("failure: Some"));
    assert!(body(&working, "second").contains("check: Some"));
}
