// Everything here fits in the digits the standard shows.
// trace: D161
import std;
import os;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $a: f64.stable = 0.5;
    let $b: f64.stable = 1.0;
    let $c: f64.stable = -0.25;
    let $d: f32.stable = ~0.1;
    std::io::fmt_print.stored(Lend.mut $out, "{}{newln}", $a);
    std::io::fmt_print.stored(Lend.mut $out, "{}{newln}", $b);
    std::io::fmt_print.stored(Lend.mut $out, "{}{newln}", $c);
    std::io::fmt_print.stored(Lend.mut $out, "{}{newln}", $d);
    return void;
end fn.main
