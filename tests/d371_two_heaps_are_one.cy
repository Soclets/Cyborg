// Every `os::heap()` is a handle to the one OS heap, so growing a list through two of them is
// fine: it really is the same memory.
// trace: D322, D332, D371
import std;
import os;

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $heap: os::heap = os::heap();
    let $other: os::heap = os::heap();
    let mut $xs: std::list<i32>[len.max: inf items] = empty;
    defer std::list.done(Give $xs, alloc: Lend $heap);
    $xs.std::%%append(val: 4, alloc: Lend $heap);
    $xs.std::%%append(val: 5, alloc: Lend $other);
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $xs.len);
    return void;
end fn.main
