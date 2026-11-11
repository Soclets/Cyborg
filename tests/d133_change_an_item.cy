// trace: D133, D125
import std;
import os;

fn set_first [$xs: Lent.mut std::array<i32>[len: 2 items], $to: i32] returns void:
    change $xs[pos: 1] set_to $to;
    return void;
end fn set_first

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let mut $a: std::array<i32>[len: 2 items] set_to [1, 2];
    change $a[pos: 2] set_to 9;
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $a[offset.items: 1]);
    set_first(Lend.mut $a, 5);
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $a[pos: 1]);
    change $a[offset.items: 0] set_to $a[pos: 1] + $a[pos: 2];
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $a[pos: 1]);
    return void;
end fn.main
