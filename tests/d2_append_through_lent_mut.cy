// trace: D2, D4
import std;
import os;

fn add [$xs: Lent.mut std::coll::list<i32>[len.max: inf items], $heap: Lent os::heap] -> void:
    $xs.std::%%append(val: 1, alloc: Lend $heap);
    return void;
end fn add

fn.main [none] -> void:
    let $heap: os::heap = os::heap();
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let mut $xs: std::coll::list<i32>[len.max: inf items] = empty;
    defer std::coll::list.done(Give $xs, alloc: Lend $heap);
    add(Lend.mut $xs, Lend $heap);
    add(Lend.mut $xs, Lend $heap);
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $xs.len);
    return void;
end fn.main
