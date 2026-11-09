// trace: D216, D83
@config(default);
import std;

const $max: i32 = 100;
const $twice := $max x 2;

fn.main [] -> void:
    const $half := $max div.trunc 2;
    std::io::fmt_print("{} {}{newln}", $twice, $half);
    return void;
end fn.main
