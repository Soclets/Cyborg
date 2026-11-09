// trace: D116, D44
import std;
@config(default);

fn read [] -> i32:
    fails [none];

    fail none;
end fn read

fn.main [] -> void:
    let $a: i32 = 1;
    let $b: i32 = $a + (read() ?? 5);
    std::io::fmt_print("{}{newln}", $b);
    return void;
end fn.main
