// D173's example, which D184 made writable: two number fields, each lent to change.
// trace: D184, D173
import std;
import os;

struct reading:
    celsius: i32;
    time: i32;
end struct reading

fn adjust [$c: Lent.mut i32, $t: Lent.mut i32] returns void:
    change $c set_to $c + 1;
    change $t set_to $t x 2;
    return void;
end fn adjust

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let mut $r: reading set_to reading[celsius set_to 21, time set_to 3600];
    adjust(Lend.mut $r.celsius, Lend.mut $r.time);
    std::io::fmt_print(Lend.mut $out, "{} {}{newln}", $r.celsius, $r.time);
    return void;
end fn.main
