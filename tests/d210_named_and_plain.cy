// trace: D210
@config(default);
import std;

fn.main [] -> void:
    let $t: i32 = 21;
    std::io::fmt_print("{$t} {}{newln}", 3);
    return void;
end fn.main
