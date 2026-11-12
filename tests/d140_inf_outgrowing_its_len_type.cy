// `inf` always fits as a limit, but what happens when the count can't be stored isn't stated.
// trace: D140, D96
import std;
import os;

fn.main [none] -> void:
    let $heap: os::heap = os::heap();
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let mut $xs: std::coll::list<i32>[len.max: inf items, len.type: u8] = empty;
    defer std::coll::list.done(Give $xs, alloc: Lend $heap);
    for $i in range[from: 1, to: 255]:
        $xs.std::%%append(val: $i, alloc: Lend $heap);
    end for
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $xs.len);
    $xs.std::%%append(val: 256, alloc: Lend $heap);
    std::io::fmt_print(Lend.mut $out, "after{newln}");
    return void;
end fn.main
