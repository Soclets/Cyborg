// A plain `i32` means just a number.
// trace: D101
@config(default);
import std;

fn.main [] -> void:
    let $x: i32 = 3600;
    std::io::fmt_print("{}{newln}", $x);
    return void;
end fn.main
