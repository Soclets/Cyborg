// trace: D20
import std;

fn.main [none] -> void:
    let $a: i32[min: 0, max: 100] = 7;
    let $b: i32 = $a;
    std::io::fmt_print("{}{newln}", $b);
    return void;
end fn.main
