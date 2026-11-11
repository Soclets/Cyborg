// trace: D161
import std;
import os;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $x: f64.stable = ~0.1;
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $x);
    return void;
end fn.main
