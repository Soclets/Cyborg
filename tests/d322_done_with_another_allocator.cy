// `done` frees a list with the allocator it came from, which is checked too.
// trace: D322, D323
import std;
import os;

fn.main [none] -> void:
    let $heap: os::heap = os::heap();
    let $other: os::heap = os::heap();
    let mut $xs: std::list<i32>[len.max: inf items] = empty;
    $xs.std::%%append(val: 4, alloc: Lend $heap);
    std::list.done(Give $xs, alloc: Lend $other);
    return void;
end fn.main
