// Comments in a manifest are `//`, on their own line or trailing.
// trace: D230
import std;
import os;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    std::io::fmt_print(Lend.mut $out, "hi{newln}");
    return void;
end fn.main
