// trace: D194
import std;
@config(default);

fn.main [] -> void:
    let $x: f64 = 2.5;
    std::io::fmt_print("{base: hex}{newln}", $x);
    return void;
end fn.main
