// Lists, maps and arrays live in `std::coll`, and `use` shortens any of them.
// trace: D378, D196
import std;
import os;
use std::coll::list as list;

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $heap: os::heap = os::heap();
    let $xs: list<i32>[len.max: 4 items] = [1, 2, 3];
    let $ys: std::coll::array<i32>[len: 2 items] = [4, 5];
    let mut $zs: std::coll::list<i32>[len.max: inf items] = empty;
    defer std::coll::list.done(Give $zs, alloc: Lend $heap);
    $zs.std::%%append(val: 6, alloc: Lend $heap);
    std::io::fmt_print(Lend.mut $out, "{} {} {}{newln}", $xs.len, $ys.len, $zs.len);
    return void;
end fn.main
