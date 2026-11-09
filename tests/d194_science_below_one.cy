// trace: D194, D165
import std;
@config(default);

fn.main [] -> void:
    let $x: f64 = 0.5;
    std::io::fmt_print("{form: science}{newln}", $x);
    return void;
end fn.main
