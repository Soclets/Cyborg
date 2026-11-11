// trace: D133, D2
import std;
import os;

fn show [$s: str] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $s);
    return void;
end fn show

fn.main [none] returns void:
    let $xs: std::array<str>[len: 2 items] set_to ["a", "b"];
    let $l: Lend from $xs str set_to Lend $xs[pos: 2];
    show(Lend $l);
    return void;
end fn.main
