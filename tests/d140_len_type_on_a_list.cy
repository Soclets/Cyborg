// trace: D140
import std;
import os;

fn total [$xs: std::list<i32>[len.max: inf items, len.type: u32]] returns u32[unit: items]:
    return $xs.len;
end fn total

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let mut $xs: std::list<i32>[len.max: inf items, len.type: u32] set_to empty;
    $xs.std::%%append(val: 1);
    let $n: u32[unit: items] set_to $xs.len;
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $n);
    std::io::fmt_print(Lend.mut $out, "{}{newln}", total(Lend $xs));
    let $small: std::list<i32>[len.min: 1 items, len.max: 8 items, len.type: u8] set_to [1, 2];
    let $s: u8[unit: items] set_to $small.len;
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $s);
    return void;
end fn.main
