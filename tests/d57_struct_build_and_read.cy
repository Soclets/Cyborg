// trace: D57
import std;
import os;

struct reading:
    celsius: i32;
    label: str;
end struct reading

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $r: reading = reading[celsius = 21, label = "kitchen"];
    std::io::fmt_print(Lend.mut $out, "{}: {}{newln}", $r.label, $r.celsius);
    return void;
end fn.main
