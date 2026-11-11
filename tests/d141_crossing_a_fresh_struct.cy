// A fresh value has no holder, so nothing's left behind.
// trace: D141, D2
import std;
import os;

struct reading:
    celsius: i32;
end struct reading

type.new wrapped = reading;

fn make [none] returns wrapped:
    return reading[celsius = 4] as wrapped;
end fn make

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $r: reading = make() as reading;
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $r.celsius);
    return void;
end fn.main
