// `grow:` is `exact`, or `by` a size.
// trace: D385
import std;
import os;

fn.main [none] -> void:
    let $heap: os::heap = os::heap();
    let mut $a: std::coll::list<i32>[len.max: inf items, grow: fast] = empty;
    defer std::coll::list.done(Give $a, alloc: Lend $heap);
    return void;
end fn.main
