// trace: D164, D83
@config(default);
import std;

const $half: f64.stable = 1.0 div 2.0;

fn.main [] -> void:
    std::io::fmt_print("{}{newln}", $half);
    return void;
end fn.main
