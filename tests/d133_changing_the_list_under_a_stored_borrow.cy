// trace: D133, D2
import std;

fn.main [none] -> void:
    let mut $xs: std::coll::array<str[len.max: 32 bytes]>[len: 2 items] = ["a", "b"];
    let $l: Lend from $xs str[len.max: 32 bytes] = Lend $xs[pos: 2];
    change $xs = ["c", "d"];
    let $k: str[len.max: 32 bytes] = Copy $l;
    return void;
end fn.main
