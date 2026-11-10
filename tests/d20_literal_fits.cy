// trace: D20
import std;

fn.main [none] -> void:
    let $a: i32[min: 0, max: 100] = 50;
    std::io::fmt_print("{}{newln}", $a);
    return void;
end fn.main
