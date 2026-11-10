// trace: D141, D20
import std;

type.new percent = i32[min: 0, max: 100];

fn.main [none] -> void:
    let $p: percent = 40;
    let $n: i32 = $p as i32;
    std::io::fmt_print("{}{newln}", $n);
    return void;
end fn.main
