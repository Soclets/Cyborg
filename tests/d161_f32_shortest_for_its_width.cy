// trace: D161
@config(default);
import std;

fn.main [] -> void:
    let $x: f32 = ~0.1;
    std::io::fmt_print("{}{newln}", $x);
    return void;
end fn.main
