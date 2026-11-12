// A list with a finite `len.max` holds its items inside it, so it takes no allocator.
// trace: D321
import std;
import os;

fn.main [none] -> void:
    let $heap: os::heap = os::heap();
    let mut $xs: std::list<i32>[len.max: 4 items] = empty;
    $xs.std::%%append(val: 4, alloc: Lend $heap);
    return void;
end fn.main
