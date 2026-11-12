// trace: D178
import std;
import os;

use std::list.%remove as %remove;

fn.main [none] -> void:
    let $heap: os::heap = os::heap();
    let mut $xs: std::list<i32>[len.max: inf items] = empty;
    defer std::list.done(Give $xs, alloc: Lend $heap);
    $xs.%remove(pos: 1);
    return void;
end fn.main
