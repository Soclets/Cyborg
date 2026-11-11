// trace: D93
import std;
import os;

use std::io::fmt_print as "say";

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    say(Lend.mut $out, "hi{newln}");
    return void;
end fn.main
