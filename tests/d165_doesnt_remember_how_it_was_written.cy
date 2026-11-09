// trace: D165, D162
@config(default);
import std;

fn.main [] -> void:
    let $a: f64 = exponent ~1.0e30;
    std::io::fmt_print.stored("{}{newln}", $a);
    return void;
end fn.main
