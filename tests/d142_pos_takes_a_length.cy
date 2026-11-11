// trace: D142, D143
import std;
import os;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let mut $xs: std::list<i32>[len.max: 8 items] = [10, 20, 30];
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $xs[pos: $xs.len]);
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $xs[pos: 3]);
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $xs[pos: 3 items]);
    change $xs[pos: $xs.len] = 99;
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $xs[pos: 3]);
    let $i: u64[unit: items] = 2 items;
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $xs[pos: $i]);
    return void;
end fn.main
