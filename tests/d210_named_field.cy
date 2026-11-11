// trace: D210, D247
import std;
import os;

struct reading:
    celsius: i32;
end struct reading

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $r: reading = reading[celsius = 3];
    std::io::fmt_print(Lend.mut $out, "{$r.celsius}{newln}");
    return void;
end fn.main
