// trace: D180, D2
import std;
import os;

struct reading:
    celsius: i32;
end struct reading

fn warm_all [$rs: Lent.mut std::list<reading>[len.max: inf items]] returns void:
    for $r in Lend.mut $rs:
        change $r.celsius = $r.celsius + 5;
    end for
    return void;
end fn warm_all

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let mut $readings: std::list<reading>[len.max: inf items] = [reading[celsius = 1]];
    warm_all(Lend.mut $readings);
    for $r in $readings:
        std::io::fmt_print(Lend.mut $out, "{}{newln}", $r.celsius);
    end for
    return void;
end fn.main
