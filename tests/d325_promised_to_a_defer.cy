// A value a `defer` will give away stays yours to read and change, but not to give away.
// trace: D325
import std;
import os;

fn eat [$xs: Given std::list<i32>[len.max: inf items], $heap: os::heap] -> void:
    std::list.done(Give $xs, alloc: Lend $heap);
    return void;
end fn eat

fn.main [none] -> void:
    let $heap: os::heap = os::heap();
    let mut $xs: std::list<i32>[len.max: inf items] = empty;
    defer std::list.done(Give $xs, alloc: Lend $heap);
    $xs.std::%%append(val: 4, alloc: Lend $heap);
    eat(Give $xs, Lend $heap);
    return void;
end fn.main
