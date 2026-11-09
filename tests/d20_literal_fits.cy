// trace: D20
import std;
@config(default);

fn.main [] -> void:
    let $a: i32[min: 0, max: 100] = 50;
    std::io::fmt_print("{}{newln}", $a);
    return void;
end fn.main
