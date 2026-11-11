// trace: D133, D86
import std;
import os;

fn.pure first [$xs: std::array<i32>[len: 2 items]] returns i32:
    return $xs[pos: 1];
end fn.pure first

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $a: std::array<i32>[len: 2 items] = [6, 7];
    std::io::fmt_print(Lend.mut $out, "{}{newln}", first(Lend $a));
    return void;
end fn.main
