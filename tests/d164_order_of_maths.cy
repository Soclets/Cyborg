// trace: D164, D116
import std;
import os;

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $a: f64.stable = 1.0 + 4.0 div 2.0;
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $a);
    let $b: f64.stable = (1.0 + 4.0) div 2.0;
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $b);
    let $c: f64.stable = -4.0 div 2.0;
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $c);
    return void;
end fn.main
