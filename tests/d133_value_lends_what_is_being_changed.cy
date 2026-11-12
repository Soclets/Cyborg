// The item's place is worked out first, so nothing says what this does if the value moves it.
// trace: D133, D2
import std;
import os;

fn grow [$xs: Lent.mut std::coll::list<i32>[len.max: inf items]] -> i32:
    return 1;
end fn grow

fn.main [none] -> void:
    let $heap: os::heap = os::heap();
    let mut $xs: std::coll::list<i32>[len.max: inf items] = std::coll::list.from([1], alloc: Lend $heap);
    defer std::coll::list.done(Give $xs, alloc: Lend $heap);
    change $xs[pos: 1] = grow(Lend.mut $xs);
    return void;
end fn.main
