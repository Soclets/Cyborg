// trace: D134, D120
import std;

fn.main [none] -> void:
    let $a: first = 3;
    std::io::fmt_print("{}{newln}", $a);
    return void;
end fn.main

type.alias first = second;
type.alias second = third;
type.alias third = i32;
