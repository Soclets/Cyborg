// trace: D43, D116
import std;

fn.main [none] -> void:
    let $a: i32 = 2 + 3 x 4;
    let $b: i32 = (2 + 3) x 4;
    let $c: i32 = 10 - 3 - 2;
    std::io::fmt_print("{}{newln}", $a);
    std::io::fmt_print("{}{newln}", $b);
    std::io::fmt_print("{}{newln}", $c);
    return void;
end fn.main
