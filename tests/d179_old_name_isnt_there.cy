// `%push` is the short name now, and `%append` alone was never given one.
// trace: D179, D115
import std;

use std::list.%%append as %%"push";

fn.main [none] returns void:
    let mut $xs: std::list<i32>[len.max: inf items] set_to empty;
    $xs.%append(val: 4);
    return void;
end fn.main
