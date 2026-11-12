// After a `%` the value is always still alive.
// trace: D5
import std;
import os;

fn.main [none] -> void:
    let $heap: os::heap = os::heap();
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let mut $xs: std::list<i32>[len.max: inf items] = empty;
    defer std::list.done(Give $xs, alloc: Lend $heap);
    $xs.std::%%append(val: 1, alloc: Lend $heap);
    $xs.std::%%append(val: 2, alloc: Lend $heap);
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $xs.len);
    return void;
end fn.main
