// trace: D194, D165
@config(default);
import std;

fn.main [] -> void:
    let $x: f64.stable = 0.5;
    std::io::fmt_print("{form: science}{newln}", $x);
    return void;
end fn.main
