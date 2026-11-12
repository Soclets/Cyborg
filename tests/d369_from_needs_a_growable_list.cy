// `std::coll::list.from` makes a list that grows on the heap, so it can't go into a bounded one,
// which takes `[ ]` directly.
// trace: D369
import std;
import os;

fn.main [none] -> void:
    let $heap: os::heap = os::heap();
    let $xs: std::coll::list<i32>[len.max: 4 items] = std::coll::list.from([1, 2], alloc: Lend $heap);
    return void;
end fn.main
