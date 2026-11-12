// Which lists are accepted where another is asked for isn't stated.
// trace: D140
import std;
import os;

fn f [$xs: std::coll::list<i32>[len.max: inf items]] -> void:
    return void;
end fn f

fn.main [none] -> void:
    let $heap: os::heap = os::heap();
    let $xs: std::coll::list<i32>[len.max: inf items, len.type: u32] = empty;
    defer std::coll::list.done(Give $xs, alloc: Lend $heap);
    f(Lend $xs);
    return void;
end fn.main
