// Errors in different functions all show.
// trace: D207, D215
import std;
import os;

fn half [$n: i32] returns i32:
    let $a: i64 = $n;
    return $n div.trunc 2;
end fn half

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $b: f64.stable = 0.1;
    std::io::fmt_print(Lend.mut $out, "{}{newln}", half(4));
    return void;
end fn.main
