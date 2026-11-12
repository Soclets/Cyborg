// trace: D140
import std;
import os;

fn total [$xs: Lent std::coll::list<i32>[len.max: inf items, len.type: u32]] -> u32[unit: items]:
    return $xs.len;
end fn total

fn.main [none] -> void:
    let $heap: os::heap = os::heap();
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let mut $xs: std::coll::list<i32>[len.max: inf items, len.type: u32] = empty;
    defer std::coll::list.done(Give $xs, alloc: Lend $heap);
    $xs.std::%%append(val: 1, alloc: Lend $heap);
    let $n: u32[unit: items] = $xs.len;
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $n);
    std::io::fmt_print(Lend.mut $out, "{}{newln}", total(Lend $xs));
    let $small: std::coll::list<i32>[len.min: 1 items, len.max: 8 items, len.type: u8] = [1, 2];
    let $s: u8[unit: items] = $small.len;
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $s);
    return void;
end fn.main
