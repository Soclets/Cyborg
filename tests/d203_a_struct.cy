// trace: D203, D3
import std;
import os;

struct reading:
    celsius: i32;
end struct reading

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    decl $r: reading;
    assign $r = reading[celsius = 5];
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $r.celsius);
    return void;
end fn.main
