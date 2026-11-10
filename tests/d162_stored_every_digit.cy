// trace: D162, D161
@config(default);
import std;

fn.main [] -> void:
    let $x: f64.stable = ~0.1;
    std::io::fmt_print("{}{newln}", $x);
    std::io::fmt_print.stored("{}{newln}", $x);
    return void;
end fn.main
