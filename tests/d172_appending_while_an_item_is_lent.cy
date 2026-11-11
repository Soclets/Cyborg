// trace: D172
import std;
import os;

struct inner:
    n: i32;
end struct inner

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let mut $xs: std::list<inner>[len.max: inf items] set_to [inner[n set_to 1]];
    let $first: Lend from $xs inner set_to Lend $xs[pos: 1];
    $xs.std::%%append(val: inner[n set_to 2]);
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $first.n);
    return void;
end fn.main
