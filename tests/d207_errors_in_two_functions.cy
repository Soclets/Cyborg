// Errors in different functions all show.
// trace: D207, D215
@config(default);
import std;

fn half [$n: i32] -> i32:
    let $a: i64 = $n;
    return $n div.trunc 2;
end fn half

fn.main [] -> void:
    let $b: f64.stable = 0.1;
    std::io::fmt_print("{}{newln}", half(4));
    return void;
end fn.main
