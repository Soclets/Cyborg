// `for` reads in place, and `Lend.mut` means nothing else uses it meanwhile.
// trace: D2
import std;
import os;

fn.main [none] -> void:
    let $heap: os::heap = os::heap();
    let mut $xs: std::coll::list<i32>[len.max: inf items] = std::coll::list.from([1, 2], alloc: Lend $heap);
    defer std::coll::list.done(Give $xs, alloc: Lend $heap);
    for $x in Lend $xs:
        $xs.std::%%append(val: $x, alloc: Lend $heap);
    end for
    return void;
end fn.main
