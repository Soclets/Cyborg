// Building shows every error, not just the first.
// trace: D207, D215
@config(default);
import std;

fn.main [] -> void:
    let $a: f64 = 0.1;
    let $b: i32 = 1;
    let $c: i64 = $b;
    let $d: bool = $b < "x";
    return void;
end fn.main
