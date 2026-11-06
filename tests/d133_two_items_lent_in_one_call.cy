// `Lend.mut` on two items of one list may or may not touch, like two fields.
// trace: D133, D2
import std;

fn swap_them [$a: Lent.mut str, $b: Lent.mut str] -> void:
    return void;
end fn swap_them

fn.main [] -> void:
    let mut $xs: std::array<str>[len: 2 items] = ["a", "b"];
    swap_them(Lend.mut $xs[pos: 1], Lend.mut $xs[pos: 2]);
    return void;
end fn.main
