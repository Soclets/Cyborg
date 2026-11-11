// trace: D174, D2
import std;
import os;

struct reading:
    celsius: i32;
end struct reading

fn replace [$r: Lent.mut reading] returns void:
    change $r = reading[celsius = 0];
    return void;
end fn replace

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let mut $r: reading = reading[celsius = 21];
    let $b: Lend.mut from $r reading = Lend.mut $r;
    replace(Lend.mut $b);
    change $b.celsius = $b.celsius + 30;
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $r.celsius);
    return void;
end fn.main
