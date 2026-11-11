// trace: D198, D93
import std::io::fmt_print;
import os;
use std::io::fmt_print as fmt_print;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    fmt_print(Lend.mut $out, "hi{newln}");
    return void;
end fn.main
