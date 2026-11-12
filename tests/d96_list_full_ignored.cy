// trace: D96, D13
import std;
import os;

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let mut $xs: std::coll::list<i32>[len.max: 1 items] = empty;
    $xs.std::%%append(val: 1) ?? void;
    $xs.std::%%append(val: 2) ?? void;
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $xs.len);
    return void;
end fn.main
