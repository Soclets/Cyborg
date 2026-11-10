// trace: D161
import std;

fn.main [none] -> void:
    let $x: f32.stable = ~0.1;
    std::io::fmt_print("{}{newln}", $x);
    return void;
end fn.main
