// `.stored` is on the whole print.
// trace: D194, D162
import std;
import os;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $x: f64.stable = ~0.1;
    let $y: f64.stable = 0.5;
    std::io::fmt_print.stored(Lend.mut $out, "{} and {}{newln}", $x, $y);
    return void;
end fn.main
