// trace: D161, D159
@config(default);
import std;

fn.main [] -> void:
    let $r: f64.ieee = 1.5;
    std::io::fmt_print("{}{newln}", $r);
    return void;
end fn.main
