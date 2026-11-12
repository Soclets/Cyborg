// `grow: exact` takes only what each append needs, so 12 bytes hold three `i32`s and no more.
// trace: D385, D389
import std;
import os;

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $heap: os::heap = os::heap();
    let $arena: std::mem::arena = std::mem::arena.new(room: 12 bytes, alloc: Lend $heap);
    defer std::mem::arena.done(Give $arena, alloc: Lend $heap);
    let mut $xs: std::coll::list<i32>[len.max: inf items, grow: exact] = empty;
    defer std::coll::list.done(Give $xs, alloc: Lend $arena);
    $xs.std::%%append(val: 1, alloc: Lend $arena);
    $xs.std::%%append(val: 2, alloc: Lend $arena);
    $xs.std::%%append(val: 3, alloc: Lend $arena);
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $xs.len);
    $xs.std::%%append(val: 4, alloc: Lend $arena);
    return void;
end fn.main
