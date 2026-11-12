// Whether a `fn.pure` may allocate isn't decided.
// trace: D86, D321
import std;
import os;

fn.pure f [$heap: Lent os::heap] -> void:
    let mut $xs: std::coll::list<i32>[len.max: inf items] = empty;
    $xs.std::%%append(val: 1, alloc: Lend $heap);
    std::coll::list.done(Give $xs, alloc: Lend $heap);
    return void;
end fn.pure f

fn.main [none] -> void:
    return void;
end fn.main
