// trace: D144, D143
import std;
import os;

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $xs: std::array<i32>[len: 3 items] = [4, 5, 6];
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $xs[offset.items: 0]);
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $xs[offset.items: 1 items]);
    let $n: u64[unit: items] = 2 items;
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $xs[offset.items: $n]);
    let $i: i32 = 1;
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $xs[offset.items: $i]);
    return void;
end fn.main
