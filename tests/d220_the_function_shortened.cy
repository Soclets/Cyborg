// Shorten the function, and write the narrowing after it.
// trace: D220, D150
import std;
import os;

use std::io::fmt_print as fmt_print;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let $tenth: f64.stable set_to ~0.1;
    fmt_print.stored(Lend.mut $out, "{}{newln}", $tenth);
    return void;
end fn.main
