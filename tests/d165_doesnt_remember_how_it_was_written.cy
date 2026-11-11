// trace: D165, D162
import std;
import os;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let $a: f64.stable set_to exponent ~1.0e30;
    std::io::fmt_print.stored(Lend.mut $out, "{}{newln}", $a);
    return void;
end fn.main
