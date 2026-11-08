// trace: D178, D115
import std;

use std::list.%append as %append;

fn.main [] -> void:
    let mut $xs: std::list<i32>[len.max: inf items] = empty;
    $xs.std::%append(val: 4);
    std::println($xs.len as str);
    return void;
end fn.main
