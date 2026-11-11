// After a `%` the value is always still alive.
// trace: D5
import std;
import os;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let mut $xs: std::list<i32>[len.max: inf items] set_to empty;
    $xs.std::%%append(val: 1);
    $xs.std::%%append(val: 2);
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $xs.len);
    return void;
end fn.main
