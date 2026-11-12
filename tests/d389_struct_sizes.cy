// A struct's size comes from its layout: a bare `struct` pads each field to its alignment, so
// `u8, u64, u32` takes 24 bytes, while `struct.compact` sorts them, taking 16. Five of each
// need 120 and 80 bytes.
// trace: D319, D388, D389, D370
import std;
import os;

struct written:
    flag: u8;
    when: u64;
    level: u32;
end struct written

struct.compact sorted:
    flag: u8;
    when: u64;
    level: u32;
end struct.compact sorted

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $heap: os::heap = os::heap();
    let $one: std::mem::arena = std::mem::arena.new(room: 100 bytes, alloc: Lend $heap);
    defer std::mem::arena.done(Give $one, alloc: Lend $heap);
    let $two: std::mem::arena = std::mem::arena.new(room: 100 bytes, alloc: Lend $heap);
    defer std::mem::arena.done(Give $two, alloc: Lend $heap);
    let mut $ss: std::coll::list<sorted>[len.max: inf items] = std::coll::list.from([sorted[flag = 1, when = 2, level = 3], sorted[flag = 1, when = 2, level = 3], sorted[flag = 1, when = 2, level = 3], sorted[flag = 1, when = 2, level = 3], sorted[flag = 1, when = 2, level = 3]], alloc: Lend $two);
    defer std::coll::list.done(Give $ss, alloc: Lend $two);
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $ss.len);
    let mut $ws: std::coll::list<written>[len.max: inf items] = std::coll::list.from([written[flag = 1, when = 2, level = 3], written[flag = 1, when = 2, level = 3], written[flag = 1, when = 2, level = 3], written[flag = 1, when = 2, level = 3], written[flag = 1, when = 2, level = 3]], alloc: Lend $one);
    defer std::coll::list.done(Give $ws, alloc: Lend $one);
    return void;
end fn.main
