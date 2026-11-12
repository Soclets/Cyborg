// trace: D211, D178, D179
import std;
import os;
use std::list.%%append as %%append;
use std::list.%%append as %%"push";

fn.main [none] -> void:
    let $heap: os::heap = os::heap();
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let mut $xs: std::list<i32>[len.max: inf items] = empty;
    defer std::list.done(Give $xs, alloc: Lend $heap);
    $xs.%%append(val: 4, alloc: Lend $heap);
    $xs.%%push(val: 5, alloc: Lend $heap);
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $xs.len);
    return void;
end fn.main
