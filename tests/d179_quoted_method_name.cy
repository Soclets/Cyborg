// trace: D179
import std;
import os;

use std::list.%%append as %%"push";

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let mut $xs: std::list<i32>[len.max: inf items] = empty;
    $xs.%%push(val: 4);
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $xs.len);
    return void;
end fn.main
