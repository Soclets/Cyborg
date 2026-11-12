// trace: D133, D2
import std;

fn.main [none] -> void:
    let mut $xs: std::coll::array<str>[len: 2 items] = ["a", "b"];
    let $l: Lend from $xs str = Lend $xs[pos: 2];
    change $xs = ["c", "d"];
    let $k: str = Copy $l;
    return void;
end fn.main
