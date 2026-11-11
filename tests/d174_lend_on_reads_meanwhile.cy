// trace: D174
import std;
import os;

struct reading:
    celsius: i32;
    time: i32;
end struct reading

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let mut $r: reading = reading[celsius = 21, time = 3600];
    let $m: Lend.mut from $r reading = Lend.mut $r;
    let $n: Lend from $r reading = Lend $m;
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $m.celsius);
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $n.celsius);
    return void;
end fn.main
