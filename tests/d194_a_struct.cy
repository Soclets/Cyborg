// trace: D194
import std;
import os;

struct reading:
    celsius: i32;
end struct reading

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $r: reading = reading[celsius = 1];
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $r);
    return void;
end fn.main
