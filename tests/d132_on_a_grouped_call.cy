// trace: D132
@config(default);
import std;

fn read [] -> i32 fails [none]:
    fail none;
end fn read

fn.main [] -> void:
    let $a: i32 = (read()) ?? 4;
    std::io::fmt_print("{}{newln}", $a);
    return void;
end fn.main
