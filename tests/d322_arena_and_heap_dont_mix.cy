// Every arena is its own allocator, so a list grown in an arena can't be freed into the heap.
// trace: D322, D371, D330
import std;
import os;

fn.main [none] -> void:
    let $heap: os::heap = os::heap();
    let $arena: std::mem::arena = std::mem::arena.new(size: 64 kib, alloc: Lend $heap);
    defer std::mem::arena.done(Give $arena, alloc: Lend $heap);
    let mut $xs: std::coll::list<i32>[len.max: inf items] = std::coll::list.from([1, 2], alloc: Lend $arena);
    std::coll::list.done(Give $xs, alloc: Lend $heap);
    return void;
end fn.main
