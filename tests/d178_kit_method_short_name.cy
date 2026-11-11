// trace: D178
import std;
import os;

use std::list.%%append as %%append;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let mut $xs: std::list<i32>[len.max: inf items] = empty;
    $xs.%%append(val: 4);
    $xs.%%append(val: 5);
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $xs.len);
    return void;
end fn.main
