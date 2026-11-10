// trace: D132
import std;

fn read [none] -> i32 fails [none]:
    fail none;
end fn read

fn.main [none] -> void:
    let $a: i32 = (read()) ?? 4;
    std::io::fmt_print("{}{newln}", $a);
    return void;
end fn.main
