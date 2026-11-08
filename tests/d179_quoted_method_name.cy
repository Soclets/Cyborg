// trace: D179
import std;

use std::list.%append as %"push";

fn.main [] -> void:
    let mut $xs: std::list<i32>[len.max: inf items] = empty;
    $xs.%push(val: 4);
    std::println($xs.len as str);
    return void;
end fn.main
