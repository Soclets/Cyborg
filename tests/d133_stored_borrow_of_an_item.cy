// trace: D133, D2
import std;

fn show [$s: str] -> void:
    std::println(Lend $s);
    return void;
end fn show

fn.main [] -> void:
    let $xs: std::array<str>[len: 2 items] = ["a", "b"];
    let $l: Lend from $xs str = Lend $xs[pos: 2];
    show(Lend $l);
    return void;
end fn.main
