// Ending an arena while a list from it is alive is a failed check, so nothing dangles.
// trace: D330
import std;
import os;

fn.main [none] -> void:
    let $heap: os::heap = os::heap();
    let $arena: std::mem::arena = std::mem::arena.new(room: 64 kib, alloc: Lend $heap);
    let mut $xs: std::coll::list<i32>[len.max: inf items] = std::coll::list.from([1, 2], alloc: Lend $arena);
    defer std::coll::list.done(Give $xs, alloc: Lend $arena);
    std::mem::arena.done(Give $arena, alloc: Lend $heap);
    return void;
end fn.main
