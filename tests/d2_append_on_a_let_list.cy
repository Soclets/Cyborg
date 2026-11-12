// `%` hides `Lend.mut`, and the compiler still checks it.
// trace: D4, D3
import std;
import os;

fn.main [none] -> void:
    let $heap: os::heap = os::heap();
    let $xs: std::list<i32>[len.max: inf items] = empty;
    defer std::list.done(Give $xs, alloc: Lend $heap);
    $xs.std::%%append(val: 1, alloc: Lend $heap);
    return void;
end fn.main
