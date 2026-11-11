// A short name for the print keeps its narrowings.
// trace: D199, D197, D150
import std;
import os;

use std::io::fmt_print as fmt_print;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let $x: f64.stable set_to 0.5;
    fmt_print.stored(Lend.mut $out, "{}{newln}", $x);
    return void;
end fn.main
