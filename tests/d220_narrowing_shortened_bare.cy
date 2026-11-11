// A function's narrowing isn't shortened to its bare name, since `stored(…)` alone reads like something else.
// trace: D220, D150
import std;
import os;

use std::io::fmt_print.stored as stored;

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    stored(Lend.mut $out, "{}{newln}", ~0.1);
    return void;
end fn.main
