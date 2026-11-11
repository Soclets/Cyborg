// trace: D211, D178, D179
import std;
import os;
use std::list.%%append as %%append;
use std::list.%%append as %%"push";

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let mut $xs: std::list<i32>[len.max: inf items] set_to empty;
    $xs.%%append(val: 4);
    $xs.%%push(val: 5);
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $xs.len);
    return void;
end fn.main
