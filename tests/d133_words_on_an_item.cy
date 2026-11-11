// trace: D133, D2
import std;
import os;

fn show [$s: str] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $s);
    return void;
end fn show

fn keep [$s: Given str] returns str:
    return Give $s;
end fn keep

fn replace [$s: Lent.mut str] returns void:
    change $s set_to "changed";
    return void;
end fn replace

fn.main [none] returns void:
    let mut $a: std::array<str>[len: 2 items] set_to ["one", "two"];
    show(Lend $a[pos: 1]);
    let $k: str set_to keep(Copy $a[pos: 2]);
    show(Lend $k);
    replace(Lend.mut $a[offset.items: 0]);
    show(Lend $a[pos: 1]);
    let $c: str set_to Copy $a[pos: 2];
    show(Lend $c);
    change $a[pos: 2] set_to Give $c;
    return void;
end fn.main
