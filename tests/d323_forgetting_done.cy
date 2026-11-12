// A list that can grow on the heap is `must_Give`, so forgetting to end it doesn't compile.
// trace: D323, D325
import std;
import os;

fn.main [none] -> void:
    let $heap: os::heap = os::heap();
    let mut $xs: std::coll::list<i32>[len.max: inf items] = empty;
    $xs.std::%%append(val: 4, alloc: Lend $heap);
    return void;
end fn.main
