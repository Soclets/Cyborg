// trace: D211, D115
import std;
import os;

fn.main [none] -> void:
    let $heap: os::heap = os::heap();
    let mut $xs: std::list<i32>[len.max: inf items] = empty;
    defer std::list.done(Give $xs, alloc: Lend $heap);
    $xs.std::%append(val: 4, alloc: Lend $heap);
    return void;
end fn.main
