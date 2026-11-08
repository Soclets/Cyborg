// trace: D178
import std;

use std::list.%append as %append;

fn.main [] -> void:
    let mut $xs: std::list<i32>[len.max: inf items] = empty;
    $xs.%append(val: 4);
    $xs.%append(val: 5);
    std::println($xs.len as str);
    return void;
end fn.main
