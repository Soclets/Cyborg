// trace: D140
import std;
@config(default);

fn total [$xs: std::list<i32>[len.max: inf items, len.type: u32]] -> u32.items:
    return $xs.len;
end fn total

fn.main [] -> void:
    let mut $xs: std::list<i32>[len.max: inf items, len.type: u32] = empty;
    $xs.std::%append(val: 1);
    let $n: u32.items = $xs.len;
    std::io::fmt_print("{}{newln}", $n);
    std::io::fmt_print("{}{newln}", total(Lend $xs));
    let $small: std::list<i32>[len.min: 1 items, len.max: 8 items, len.type: u8] = [1, 2];
    let $s: u8.items = $small.len;
    std::io::fmt_print("{}{newln}", $s);
    return void;
end fn.main
