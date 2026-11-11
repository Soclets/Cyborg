// Building shows every error, not just the first.
// trace: D207, D215
import std;

fn.main [none] returns void:
    let $a: f64.stable set_to 0.1;
    let $b: i32 set_to 1;
    let $c: i64 set_to $b;
    let $d: bool set_to $b below "x";
    return void;
end fn.main
