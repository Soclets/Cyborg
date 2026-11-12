// `grow:` is written on the type, and whether a list that grows exactly is taken where a
// doubling one is asked for isn't decided, as with other list types that differ.
// trace: D385
import std;
import os;

fn count [$xs: Lent.mut std::coll::list<i32>[len.max: inf items]] -> void:
    return void;
end fn count

fn.main [none] -> void:
    let $heap: os::heap = os::heap();
    let mut $a: std::coll::list<i32>[len.max: inf items, grow: exact] = empty;
    defer std::coll::list.done(Give $a, alloc: Lend $heap);
    count(Lend.mut $a);
    return void;
end fn.main
