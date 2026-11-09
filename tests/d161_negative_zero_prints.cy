// trace: D161, D160
import std;
@config(default);

fn.main [] -> void:
    let $z: f64 = -0.0;
    std::io::fmt_print("{}{newln}", $z);
    return void;
end fn.main
