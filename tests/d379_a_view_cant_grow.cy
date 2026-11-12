// A `.mut` view is a window onto someone else's list, so it can't grow or shrink.
// trace: D379
import std;
import os;

fn grow [$xs: std::coll::list.mut<i32>] -> void:
    let $heap: os::heap = os::heap();
    $xs.std::%%append(val: 4, alloc: Lend $heap);
    return void;
end fn grow

fn.main [none] -> void:
    return void;
end fn.main
