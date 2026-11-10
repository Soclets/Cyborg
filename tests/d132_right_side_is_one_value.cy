// trace: D132
import std;

fn read [none] -> i32 fails [none]:
    fail none;
end fn read

fn.main [none] -> void:
    let $x: i32 = 5;
    std::io::fmt_print("{}{newln}", (read() ?? -1));
    std::io::fmt_print("{}{newln}", (read() ?? $x));
    std::io::fmt_print("{}{newln}", (read() ?? ($x + 1)));
    return void;
end fn.main
