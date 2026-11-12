// trace: D133, D86
import std;
import os;

fn.pure first [$xs: Lent std::coll::array<i32>[len: 2 items]] -> i32:
    return $xs[pos: 1];
end fn.pure first

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $a: std::coll::array<i32>[len: 2 items] = [6, 7];
    std::io::fmt_print(Lend.mut $out, "{}{newln}", first(Lend $a));
    return void;
end fn.main
