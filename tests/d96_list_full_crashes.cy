// A full list is a failure like any other, and its name isn't decided.
// trace: D96, D13
import std;

fn.main [none] -> void:
    let mut $xs: std::list<i32>[len.max: 2 items] = empty;
    $xs.std::%%append(val: 1);
    $xs.std::%%append(val: 2);
    std::io::fmt_print("two in{newln}");
    $xs.std::%%append(val: 3);
    std::io::fmt_print("not reached{newln}");
    return void;
end fn.main
