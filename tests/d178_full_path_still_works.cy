// trace: D178, D115
@config(default);
import std;

use std::list.%%append as %%append;

fn.main [] -> void:
    let mut $xs: std::list<i32>[len.max: inf items] = empty;
    $xs.std::%%append(val: 4);
    std::io::fmt_print("{}{newln}", $xs.len);
    return void;
end fn.main
