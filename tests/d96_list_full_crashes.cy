// A full list is a failure like any other, and its name isn't decided.
// trace: D96, D13
import std;
@config(default);

fn.main [] -> void:
    let mut $xs: std::list<i32>[len.max: 2 items] = empty;
    $xs.std::%append(val: 1);
    $xs.std::%append(val: 2);
    std::println("two in");
    $xs.std::%append(val: 3);
    std::println("not reached");
    return void;
end fn.main
