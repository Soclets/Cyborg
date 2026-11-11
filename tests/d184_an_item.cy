// trace: D184, D133
import std;
import os;

fn bump [$n: Lent.mut i32] returns void:
    change $n set_to $n + 1;
    return void;
end fn bump

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let mut $xs: std::array<i32>[len: 2 items] set_to [1, 2];
    bump(Lend.mut $xs[pos: 2]);
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $xs[pos: 2]);
    return void;
end fn.main
