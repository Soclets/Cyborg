// trace: D172
import std;
import os;

struct inner:
    n: i32;
end struct inner

fn.main [none] -> void:
    let $heap: os::heap = os::heap();
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $xs: std::coll::list<inner>[len.max: inf items] = std::coll::list.from([inner[n = 1], inner[n = 2]], alloc: Lend $heap);
    defer std::coll::list.done(Give $xs, alloc: Lend $heap);
    let $first: Lend from $xs inner = Lend $xs[pos: 1];
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $first.n);
    return void;
end fn.main
