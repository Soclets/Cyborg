// trace: D180, D2
import std;
import os;

fn make [none] -> std::coll::list<i32>[len.max: inf items]:
    let $heap: os::heap = os::heap();
    return std::coll::list.from([1, 2], alloc: Lend $heap);
end fn make

fn.main [none] -> void:
    for $n in Give make():
        return void;
    end for
    return void;
end fn.main
