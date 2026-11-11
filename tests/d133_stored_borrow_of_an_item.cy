// trace: D133, D2
import std;
import os;

fn show [$s: str] returns void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $s);
    return void;
end fn show

fn.main [none] returns void:
    let $xs: std::array<str>[len: 2 items] = ["a", "b"];
    let $l: Lend from $xs str = Lend $xs[pos: 2];
    show(Lend $l);
    return void;
end fn.main
