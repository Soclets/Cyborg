// trace: D211, D178
import std;
use std::list.%%append as %%append;

fn.main [none] -> void:
    let mut $xs: std::list<i32>[len.max: inf items] = empty;
    $xs.%append(val: 4);
    return void;
end fn.main
