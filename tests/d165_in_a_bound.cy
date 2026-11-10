// trace: D165, D158
@config(default);
import std;

fn.main [] -> void:
    let $x: f64.stable[min: 0.0, max: exponent 1.0e6] = exponent 2.5e5;
    std::io::fmt_print("{}{newln}", $x);
    return void;
end fn.main
