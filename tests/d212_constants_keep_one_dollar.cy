// trace: D212, D83
@config(default);
import std;

const $max: i32 = 100;

fn.main [] -> void:
    std::io::fmt_print("{}{newln}", $max);
    return void;
end fn.main
