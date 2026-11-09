// trace: D161, D159
import std;
@config(default);

fn.main [] -> void:
    let $r: f64.ieee = 1.5;
    std::io::fmt_print("{}{newln}", $r);
    return void;
end fn.main
