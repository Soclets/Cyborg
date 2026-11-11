// trace: D165, D162
import std;
import os;

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $a: f64.stable = exponent ~1.0e30;
    std::io::fmt_print.stored(Lend.mut $out, "{}{newln}", $a);
    return void;
end fn.main
