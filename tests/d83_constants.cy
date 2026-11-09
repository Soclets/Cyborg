// trace: D83, D59
@config(default);
import std;

const $b: i32 = 60 x 60;

fn.main [] -> void:
    const $c: i32 = $b + 1;
    std::io::fmt_print("{}{newln}", $b);
    std::io::fmt_print("{}{newln}", $c);
    return void;
end fn.main
