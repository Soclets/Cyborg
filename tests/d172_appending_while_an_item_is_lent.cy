// trace: D172
import std;
@config(default);

struct inner:
    n: i32;
end struct inner

fn.main [] -> void:
    let mut $xs: std::list<inner>[len.max: inf items] = [inner[n = 1]];
    let $first: Lend from $xs inner = Lend $xs[pos: 1];
    $xs.std::%append(val: inner[n = 2]);
    std::io::fmt_print("{}{newln}", $first.n);
    return void;
end fn.main
