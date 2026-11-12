// A list remembers which allocator it uses, and handing it another is a failed check.
// trace: D322, D332
import std;
import os;

fn.main [none] -> void:
    let $heap: os::heap = os::heap();
    let $other: os::heap = os::heap();
    let mut $xs: std::list<i32>[len.max: inf items] = empty;
    defer std::list.done(Give $xs, alloc: Lend $heap);
    $xs.std::%%append(val: 4, alloc: Lend $heap);
    $xs.std::%%append(val: 5, alloc: Lend $other);
    return void;
end fn.main
