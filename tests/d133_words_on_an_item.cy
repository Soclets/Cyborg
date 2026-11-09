// trace: D133, D2
import std;
@config(default);

fn show [$s: str] -> void:
    std::println(Lend $s);
    return void;
end fn show

fn keep [$s: Given str] -> str:
    return Give $s;
end fn keep

fn replace [$s: Lent.mut str] -> void:
    change $s = "changed";
    return void;
end fn replace

fn.main [] -> void:
    let mut $a: std::array<str>[len: 2 items] = ["one", "two"];
    show(Lend $a[pos: 1]);
    let $k: str = keep(Copy $a[pos: 2]);
    show(Lend $k);
    replace(Lend.mut $a[offset.items: 0]);
    show(Lend $a[pos: 1]);
    let $c: str = Copy $a[pos: 2];
    show(Lend $c);
    change $a[pos: 2] = Give $c;
    return void;
end fn.main
