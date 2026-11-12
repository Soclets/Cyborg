// trace: D23
import std;
import os;

fn.main [none] -> void:
    let $heap: os::heap = os::heap();
    let $xs: std::list< i32>[len.max: inf items] = empty;
    defer std::list.done(Give $xs, alloc: Lend $heap);
    return void;
end fn.main
