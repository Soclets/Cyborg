// trace: D161, D159
import std;
import os;

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $r: f64.ieee = 1.5;
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $r);
    return void;
end fn.main
