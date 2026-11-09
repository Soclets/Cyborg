// trace: D211, D178, D179
@config(default);
import std;
use std::list.%%append as %%append;
use std::list.%%append as %%"push";

fn.main [] -> void:
    let mut $xs: std::list<i32>[len.max: inf items] = empty;
    $xs.%%append(val: 4);
    $xs.%%push(val: 5);
    std::io::fmt_print("{}{newln}", $xs.len);
    return void;
end fn.main
