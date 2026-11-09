// trace: D178
import std;

use std::list.%append as %append;
@config(default);

fn.main [] -> void:
    let mut $xs: std::list<i32>[len.max: inf items] = empty;
    $xs.%append(val: 4);
    $xs.%append(val: 5);
    std::io::fmt_print("{}{newln}", $xs.len);
    return void;
end fn.main
