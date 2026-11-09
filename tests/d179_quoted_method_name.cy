// trace: D179
import std;

use std::list.%append as %"push";
@config(default);

fn.main [] -> void:
    let mut $xs: std::list<i32>[len.max: inf items] = empty;
    $xs.%push(val: 4);
    std::io::fmt_print("{}{newln}", $xs.len);
    return void;
end fn.main
