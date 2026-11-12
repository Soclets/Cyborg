// An arena takes one block from another allocator and hands it out. Lists from it still end
// with `done`, and it ends with `std::mem::arena.done`, which gives the block back.
// trace: D330, D333, D378, D370
import std;
import os;

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $heap: os::heap = os::heap();
    let $arena: std::mem::arena = std::mem::arena.new(size: 1 mib, alloc: Lend $heap);
    defer std::mem::arena.done(Give $arena, alloc: Lend $heap);
    let mut $xs: std::coll::list<i32>[len.max: inf items] = std::coll::list.from([1, 2], alloc: Lend $arena);
    defer std::coll::list.done(Give $xs, alloc: Lend $arena);
    $xs.std::%%append(val: 3, alloc: Lend $arena);
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $xs.len);
    return void;
end fn.main
