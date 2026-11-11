// A number with a unit prints its number, as `as str` does.
// trace: D194, D122
import std;
import os;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let $xs: std::array<i32>[len: 3 items] set_to [1, 2, 3];
    let $neg: f64.stable set_to -0.0;
    let $tenth: f64.stable set_to ~0.1;
    std::io::fmt_print(Lend.mut $out, "{} {} {}{newln}", $xs.len, $tenth, $neg);
    return void;
end fn.main
