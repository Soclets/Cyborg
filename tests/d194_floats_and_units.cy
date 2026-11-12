// A number with a unit prints its number, as `as str[len.max: 20 bytes]` does.
// trace: D194, D122
import std;
import os;

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $xs: std::array<i32>[len: 3 items] = [1, 2, 3];
    let $neg: f64.stable = -0.0;
    let $tenth: f64.stable = ~0.1;
    std::io::fmt_print(Lend.mut $out, "{} {} {}{newln}", $xs.len, $tenth, $neg);
    return void;
end fn.main
