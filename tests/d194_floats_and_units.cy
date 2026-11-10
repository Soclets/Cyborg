// A number with a unit prints its number, as `as str` does.
// trace: D194, D122
@config(default);
import std;

fn.main [] -> void:
    let $xs: std::array<i32>[len: 3 items] = [1, 2, 3];
    let $neg: f64.stable = -0.0;
    let $tenth: f64.stable = ~0.1;
    std::io::fmt_print("{} {} {}{newln}", $xs.len, $tenth, $neg);
    return void;
end fn.main
