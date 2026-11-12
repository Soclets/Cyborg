// A `str` view is a pointer and a pointer-sized length, 8 bytes on `arduino_uno_r4_wifi`, so
// room for four views takes 32 bytes there.
// trace: D390, D387, D389
import std;
import os;

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $heap: os::heap = os::heap();
    let $arena: std::mem::arena = std::mem::arena.new(room: 32 bytes, alloc: Lend $heap);
    defer std::mem::arena.done(Give $arena, alloc: Lend $heap);
    let mut $names: std::coll::list<str>[len.max: inf items] = std::coll::list.from(["hall", "attic"], alloc: Lend $arena);
    defer std::coll::list.done(Give $names, alloc: Lend $arena);
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $names.len);
    return void;
end fn.main
