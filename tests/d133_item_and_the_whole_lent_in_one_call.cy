// trace: D133, D2
@config(default);
import std;

fn both [$xs: Lent.mut std::array<str>[len: 2 items], $s: str] -> void:
    return void;
end fn both

fn.main [] -> void:
    let mut $xs: std::array<str>[len: 2 items] = ["a", "b"];
    both(Lend.mut $xs, Lend $xs[pos: 1]);
    return void;
end fn.main
