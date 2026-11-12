// A growable list doubles when it's full unless its type says `grow:`: `exact` takes only what
// each append needs, and `by` grows in fixed steps of bytes or items.
// trace: D385
import std;
import os;

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $heap: os::heap = os::heap();
    let mut $a: std::coll::list<i32>[len.max: inf items, grow: exact] = empty;
    defer std::coll::list.done(Give $a, alloc: Lend $heap);
    let mut $b: std::coll::list<i32>[len.max: inf items, grow: by 4 kib] = empty;
    defer std::coll::list.done(Give $b, alloc: Lend $heap);
    let mut $c: std::coll::list<i32>[len.max: inf items, grow: by 64 items] = std::coll::list.from([1], alloc: Lend $heap);
    defer std::coll::list.done(Give $c, alloc: Lend $heap);
    $a.std::%%append(val: 1, alloc: Lend $heap);
    $b.std::%%append(val: 2, alloc: Lend $heap);
    std::io::fmt_print(Lend.mut $out, "{} {} {}{newln}", $a.len, $b.len, $c.len);
    return void;
end fn.main
