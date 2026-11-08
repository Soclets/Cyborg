// trace: D178
import std;

use std::list.%remove as %remove;

fn.main [] -> void:
    let mut $xs: std::list<i32>[len.max: inf items] = empty;
    $xs.%remove(pos: 1);
    return void;
end fn.main
