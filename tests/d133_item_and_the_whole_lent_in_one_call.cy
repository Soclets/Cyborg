// trace: D133, D2
import std;

fn both [$xs: Lent.mut std::array<str>[len: 2 items], $s: str] returns void:
    return void;
end fn both

fn.main [none] returns void:
    let mut $xs: std::array<str>[len: 2 items] set_to ["a", "b"];
    both(Lend.mut $xs, Lend $xs[pos: 1]);
    return void;
end fn.main
