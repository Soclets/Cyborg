// trace: D164, D116
import std;
import os;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $a: f64.stable = 8.0 div 2.0 x 3.0;
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $a);
    return void;
end fn.main
