// trace: D132
@config(default);
import std;

fn read [] -> i32 fails [none]:
    fail none;
end fn read

fn.main [] -> void:
    let $x: i32 = 5;
    std::io::fmt_print("{}{newln}", (read() ?? -1));
    std::io::fmt_print("{}{newln}", (read() ?? $x));
    std::io::fmt_print("{}{newln}", (read() ?? ($x + 1)));
    return void;
end fn.main
