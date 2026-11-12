// A list grown through one `os::heap()` can be freed through another, since there's only one
// OS heap.
// trace: D322, D323, D371
import std;
import os;

fn.main [none] -> void:
    let $heap: os::heap = os::heap();
    let $other: os::heap = os::heap();
    let mut $xs: std::coll::list<i32>[len.max: inf items] = empty;
    $xs.std::%%append(val: 4, alloc: Lend $heap);
    std::coll::list.done(Give $xs, alloc: Lend $other);
    return void;
end fn.main
