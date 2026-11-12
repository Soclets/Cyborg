// `%push` is the short name now, and `%append` alone was never given one.
// trace: D179, D115
import std;
import os;

use std::list.%%append as %%"push";

fn.main [none] -> void:
    let $heap: os::heap = os::heap();
    let mut $xs: std::list<i32>[len.max: inf items] = empty;
    defer std::list.done(Give $xs, alloc: Lend $heap);
    $xs.%append(val: 4, alloc: Lend $heap);
    return void;
end fn.main
