// trace: D180, D128
import std;
import os;

struct reading:
    celsius: i32;
end struct reading

fn.main [none] -> void:
    let $heap: os::heap = os::heap();
    let mut $readings: std::coll::list<reading>[len.max: inf items] = std::coll::list.from([reading[celsius = 20]], alloc: Lend $heap);
    defer std::coll::list.done(Give $readings, alloc: Lend $heap);
    for $r in Lend $readings:
        change $r.celsius = 0;
    end for
    return void;
end fn.main
