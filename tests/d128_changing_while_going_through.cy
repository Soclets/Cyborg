// trace: D128, D2
import std;
import os;

fn.main [none] -> void:
    let $heap: os::heap = os::heap();
    let mut $xs: std::coll::list<i32>[len.max: inf items] = [1, 2];
    defer std::coll::list.done(Give $xs, alloc: Lend $heap);
    for $x in $xs:
        change $xs = [3];
    end for
    return void;
end fn.main
