// trace: D180
import std;
import os;

struct reading:
    celsius: i32;
end struct reading

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let mut $readings: std::list<reading>[len.max: inf items] set_to [reading[celsius set_to 20], reading[celsius set_to 30]];
    for $r in Lend.mut $readings:
        change $r.celsius set_to $r.celsius + 1;
    end for
    for $r in $readings:
        std::io::fmt_print(Lend.mut $out, "{}{newln}", $r.celsius);
    end for
    return void;
end fn.main
