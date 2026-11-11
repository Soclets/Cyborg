// A `rule` line is a placeholder, so it is NotDesignedYet.
// trace: D265
import std;
import os;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    std::io::fmt_print(Lend.mut $out, "hi{newln}");
    return void;
end fn.main
