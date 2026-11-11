// A quoted name for a function's narrowing is your own choice.
// trace: D220, D150
import std;
import os;

use std::io::fmt_print.stored as "print_exact";

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let $tenth: f64.stable set_to ~0.1;
    print_exact(Lend.mut $out, "{}{newln}", $tenth);
    return void;
end fn.main
