// trace: D134, D20
import std;

type.alias percent = i32[min: 0, max: 100];

fn.main [none] -> void:
    let $a: percent = 50;
    std::io::fmt_print("{}{newln}", $a);
    let $big: i32 = 101;
    let $b: percent = $big;
    return void;
end fn.main
