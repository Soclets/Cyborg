// Importing a thing doesn't shorten it, so a bare name is still nothing.
// trace: D198
import std::io::fmt_print;
import os;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    fmt_print(Lend.mut $out, "hi{newln}");
    return void;
end fn.main
