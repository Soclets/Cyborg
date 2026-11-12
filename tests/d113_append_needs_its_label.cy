// trace: D113, D114
import std;
import os;

fn.main [none] -> void:
    let $heap: os::heap = os::heap();
    let mut $xs: std::coll::list<i32>[len.max: inf items] = empty;
    defer std::coll::list.done(Give $xs, alloc: Lend $heap);
    $xs.std::%%append(4, alloc: Lend $heap);
    return void;
end fn.main
