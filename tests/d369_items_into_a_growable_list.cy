// `[ ]` straight into a list that grows on the heap is refused, since filling it allocates, and
// the fix names the allocator.
// trace: D369, D321
import std;
import os;

fn.main [none] -> void:
    let $heap: os::heap = os::heap();
    let mut $xs: std::coll::list<i32>[len.max: inf items] = [1, 2, 3];
    defer std::coll::list.done(Give $xs, alloc: Lend $heap);
    return void;
end fn.main
