// trace: D180, D2
import std;
import os;

struct reading:
    celsius: i32;
end struct reading

fn warm_all [$rs: Lent.mut std::coll::list<reading>[len.max: inf items]] -> void:
    for $r in Lend.mut $rs:
        change $r.celsius = $r.celsius + 5;
    end for
    return void;
end fn warm_all

fn.main [none] -> void:
    let $heap: os::heap = os::heap();
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let mut $readings: std::coll::list<reading>[len.max: inf items] = std::coll::list.from([reading[celsius = 1]], alloc: Lend $heap);
    defer std::coll::list.done(Give $readings, alloc: Lend $heap);
    warm_all(Lend.mut $readings);
    for $r in Lend $readings:
        std::io::fmt_print(Lend.mut $out, "{}{newln}", $r.celsius);
    end for
    return void;
end fn.main
