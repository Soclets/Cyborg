// trace: D140, D7
import std;
import os;

fn.main [none] -> void:
    let $heap: os::heap = os::heap();
    let $xs: std::list<i32>[len.max: inf items, len.type: u32] = empty;
    defer std::list.done(Give $xs, alloc: Lend $heap);
    let $n: u64[unit: items] = $xs.len;
    return void;
end fn.main
