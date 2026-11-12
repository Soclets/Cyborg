// A pointer's size is the target's, so when an arena runs out with pointers inside depends on
// it: on `macos_aarch64` a pointer takes 8 bytes, so room for four takes 32, more than 16.
// trace: D391, D387, D389, D370
import std;
import os;

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $heap: os::heap = os::heap();
    let $arena: std::mem::arena = std::mem::arena.new(room: 16 bytes, alloc: Lend $heap);
    defer std::mem::arena.done(Give $arena, alloc: Lend $heap);
    let $count: i32 = 5;
    shield pointers:
        reason "testing how much a list of pointers takes"
        let $p: ptr.raw<i32> = ptr.raw<i32>.of(Lend $count);
        let mut $ps: std::coll::list<ptr.raw<i32>>[len.max: inf items] = std::coll::list.from([$p], alloc: Lend $arena);
        std::io::fmt_print(Lend.mut $out, "{}{newln}", $ps.len);
        std::coll::list.done(Give $ps, alloc: Lend $arena);
    end shield pointers
    return void;
end fn.main
