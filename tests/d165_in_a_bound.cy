// trace: D165, D158
import std;
import os;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let $x: f64.stable[min: 0.0, max: exponent 1.0e6] set_to exponent 2.5e5;
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $x);
    return void;
end fn.main
