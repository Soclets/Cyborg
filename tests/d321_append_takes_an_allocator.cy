// A list that grows on the heap says where the memory comes from on every `append`.
// trace: D321, D322
import std;
import os;

fn.main [none] -> void:
    let $heap: os::heap = os::heap();
    let mut $xs: std::coll::list<i32>[len.max: inf items] = empty;
    defer std::coll::list.done(Give $xs, alloc: Lend $heap);
    $xs.std::%%append(val: 4);
    return void;
end fn.main
