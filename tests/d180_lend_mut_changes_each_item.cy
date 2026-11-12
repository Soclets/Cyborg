// trace: D180
import std;
import os;

struct reading:
    celsius: i32;
end struct reading

fn.main [none] -> void:
    let $heap: os::heap = os::heap();
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let mut $readings: std::coll::list<reading>[len.max: inf items] = std::coll::list.from([reading[celsius = 20], reading[celsius = 30]], alloc: Lend $heap);
    defer std::coll::list.done(Give $readings, alloc: Lend $heap);
    for $r in Lend.mut $readings:
        change $r.celsius = $r.celsius + 1;
    end for
    for $r in $readings:
        std::io::fmt_print(Lend.mut $out, "{}{newln}", $r.celsius);
    end for
    return void;
end fn.main
