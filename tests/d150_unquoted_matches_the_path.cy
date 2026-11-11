// trace: D150
import std;
import os;

use std::io::fmt_print as fmt_print;

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    fmt_print(Lend.mut $out, "hi{newln}");
    return void;
end fn.main
