// trace: D133, D2
import std;

fn both [$xs: Lent.mut std::coll::array<str>[len: 2 items], $s: str] -> void:
    return void;
end fn both

fn.main [none] -> void:
    let mut $xs: std::coll::array<str>[len: 2 items] = ["a", "b"];
    both(Lend.mut $xs, Lend $xs[pos: 1]);
    return void;
end fn.main
