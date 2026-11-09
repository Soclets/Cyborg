// trace: D161
import std;
@config(default);

fn.main [] -> void:
    let $x: f32 = ~0.1;
    std::io::fmt_print("{}{newln}", $x);
    return void;
end fn.main
