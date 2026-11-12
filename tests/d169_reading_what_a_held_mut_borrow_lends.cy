// A stored `Lend.mut` is still held while a `change` through it works out its value, so the
// value can't read what it borrows.
// trace: D2, D169
import std;
import os;

struct reading:
    celsius: i32;
end struct reading

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let mut $r: reading = reading[celsius = 21];
    let $m: Lend.mut from $r reading = Lend.mut $r;
    change $m.celsius = $r.celsius + 1;
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $r.celsius);
    return void;
end fn.main
