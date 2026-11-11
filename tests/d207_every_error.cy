// Building shows every error, not just the first.
// trace: D207, D215
import std;

fn.main [none] returns void:
    let $a: f64.stable = 0.1;
    let $b: i32 = 1;
    let $c: i64 = $b;
    let $d: bool = $b below "x";
    return void;
end fn.main
