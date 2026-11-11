// trace: D149, D3
import std;
import os;

struct reading:
    celsius: i32;
end struct reading

type.new wrapped = reading;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $r: reading = reading[celsius = 1];
    let $w: wrapped = Copy $r as wrapped;
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $r.celsius);
    return void;
end fn.main
