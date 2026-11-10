// `$first` is one item, and still all of `$xs` is locked, since adding can move every item.
// trace: D172
import std;

struct inner:
    n: i32;
end struct inner

fn.main [none] -> void:
    let mut $xs: std::list<inner>[len.max: inf items] = [inner[n = 1], inner[n = 2]];
    let $first: Lend from $xs inner = Lend $xs[pos: 1];
    change $xs[pos: 2].n = 5;
    std::io::fmt_print("{}{newln}", $first.n);
    return void;
end fn.main
