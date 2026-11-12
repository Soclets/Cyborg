// trace: D133, D2
import std;
import os;

fn show [$s: str[len.max: 32 bytes]] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $s);
    return void;
end fn show

fn.main [none] -> void:
    let $xs: std::coll::array<str[len.max: 32 bytes]>[len: 2 items] = ["a", "b"];
    let $l: Lend from $xs str[len.max: 32 bytes] = Lend $xs[pos: 2];
    show(Lend $l);
    return void;
end fn.main
