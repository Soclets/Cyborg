// trace: D194
import std;
import os;

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $x: f64.stable = 2.5;
    std::io::fmt_print(Lend.mut $out, "{base: hex}{newln}", $x);
    return void;
end fn.main
