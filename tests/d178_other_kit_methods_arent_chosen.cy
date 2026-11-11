// trace: D178
import std;

use std::list.%remove as %remove;

fn.main [none] returns void:
    let mut $xs: std::list<i32>[len.max: inf items] set_to empty;
    $xs.%remove(pos: 1);
    return void;
end fn.main
