// trace: D161, D160
import std;
import os;

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $z: f64.stable = -0.0;
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $z);
    return void;
end fn.main
