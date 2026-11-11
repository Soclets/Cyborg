// trace: D216, D3
import std;
import os;

struct reading:
    celsius: i32;
end struct reading

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $r: reading = reading[celsius = 1];
    let $s := Give $r;
    let $t := reading[celsius = 2];
    std::io::fmt_print(Lend.mut $out, "{} {}{newln}", $s.celsius, $t.celsius);
    return void;
end fn.main
