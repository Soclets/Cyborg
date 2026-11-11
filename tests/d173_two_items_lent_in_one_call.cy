// Two items of one list may be the same one, so they stay locked together.
// trace: D173, D133, D2
import std;

fn swap_them [$a: Lent.mut str, $b: Lent.mut str] -> void:
    return void;
end fn swap_them

fn.main [none] -> void:
    let mut $xs: std::array<str>[len: 2 items] = ["a", "b"];
    swap_them(Lend.mut $xs[pos: 1], Lend.mut $xs[pos: 2]);
    return void;
end fn.main
