// An arena runs out at the same point everywhere: a list of `i32` takes room for 4, then 8,
// then 16 items, growing where it is while it's the arena's last block, so 64 bytes hold 16.
// trace: D370, D385, D386, D389, D330
import std;
import os;

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $heap: os::heap = os::heap();
    let $arena: std::mem::arena = std::mem::arena.new(room: 64 bytes, alloc: Lend $heap);
    defer std::mem::arena.done(Give $arena, alloc: Lend $heap);
    let mut $xs: std::coll::list<i32>[len.max: inf items] = empty;
    defer std::coll::list.done(Give $xs, alloc: Lend $arena);
    let mut $i: i32 = 0;
    let mut $full: bool = false;
    while not $full:
        $xs.std::%%append(val: $i, alloc: Lend $arena) otherwise:
            case out_of_memory:
                change $full = true;
        end otherwise
        change $i = $i + 1;
    end while
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $xs.len);
    return void;
end fn.main
