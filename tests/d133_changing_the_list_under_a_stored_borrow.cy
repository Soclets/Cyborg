// trace: D133, D2
import std;

fn.main [none] returns void:
    let mut $xs: std::array<str>[len: 2 items] set_to ["a", "b"];
    let $l: Lend from $xs str set_to Lend $xs[pos: 2];
    change $xs set_to ["c", "d"];
    let $k: str set_to Copy $l;
    return void;
end fn.main
