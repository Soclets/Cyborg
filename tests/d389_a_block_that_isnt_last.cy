// A list's block grows where it is only while it's the arena's last one, so a list that grows
// after another list was made takes a new block, and the old one stays used.
// trace: D389, D385
import std;
import os;

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $heap: os::heap = os::heap();
    let $arena: std::mem::arena = std::mem::arena.new(room: 64 bytes, alloc: Lend $heap);
    defer std::mem::arena.done(Give $arena, alloc: Lend $heap);
    let mut $a: std::coll::list<i32>[len.max: inf items] = std::coll::list.from([1, 2, 3, 4], alloc: Lend $arena);
    defer std::coll::list.done(Give $a, alloc: Lend $arena);
    let mut $b: std::coll::list<i32>[len.max: inf items] = std::coll::list.from([5, 6, 7, 8], alloc: Lend $arena);
    defer std::coll::list.done(Give $b, alloc: Lend $arena);
    $a.std::%%append(val: 9, alloc: Lend $arena) otherwise:
        case out_of_memory:
            std::io::fmt_print(Lend.mut $out, "a ran out{newln}");
    end otherwise
    $b.std::%%append(val: 10, alloc: Lend $arena) otherwise:
        case out_of_memory:
            std::io::fmt_print(Lend.mut $out, "b ran out{newln}");
    end otherwise
    std::io::fmt_print(Lend.mut $out, "{} {}{newln}", $a.len, $b.len);
    return void;
end fn.main
