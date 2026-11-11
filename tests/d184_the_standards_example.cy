// D173's example, which D184 made writable: two number fields, each lent to change.
// trace: D184, D173
import std;
import os;

struct reading:
    celsius: i32;
    time: i32;
end struct reading

fn adjust [$c: Lent.mut i32, $t: Lent.mut i32] -> void:
    change $c = $c + 1;
    change $t = $t x 2;
    return void;
end fn adjust

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let mut $r: reading = reading[celsius = 21, time = 3600];
    adjust(Lend.mut $r.celsius, Lend.mut $r.time);
    std::io::fmt_print(Lend.mut $out, "{} {}{newln}", $r.celsius, $r.time);
    return void;
end fn.main
