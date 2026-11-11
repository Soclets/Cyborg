// trace: D197, D199, D93
import std;
import os;

use std::io::fmt_print as "say";

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $tenth: f64.stable = ~0.1;
    say(Lend.mut $out, "hi{newln}");
    say.stored(Lend.mut $out, "{}{newln}", $tenth);
    return void;
end fn.main
