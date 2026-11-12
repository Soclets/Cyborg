// Heap text comes from `str.from`, which takes an allocator, and `str.done` ends it, since it's
// `must_Give`.
// trace: D326, D328, D331
import std;
import os;

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $heap: os::heap = os::heap();
    let $n: i64 = 9227465;
    let $log: str[len.max: inf bytes] = str.from($n, alloc: Lend $heap);
    defer str.done(Give $log, alloc: Lend $heap);
    std::io::fmt_print(Lend.mut $out, "{$log}{newln}");
    return void;
end fn.main
