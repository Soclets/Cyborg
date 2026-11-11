// trace: D96, D13
import std;
import os;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let mut $xs: std::list<i32>[len.max: 1 items] set_to empty;
    $xs.std::%%append(val: 1) ?? void;
    $xs.std::%%append(val: 2) ?? void;
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $xs.len);
    return void;
end fn.main
