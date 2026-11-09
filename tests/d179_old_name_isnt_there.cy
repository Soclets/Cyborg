// `%push` is the short name now, and `%append` alone was never given one.
// trace: D179, D115
@config(default);
import std;

use std::list.%append as %"push";

fn.main [] -> void:
    let mut $xs: std::list<i32>[len.max: inf items] = empty;
    $xs.%append(val: 4);
    return void;
end fn.main
