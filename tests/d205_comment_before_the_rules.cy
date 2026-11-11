// trace: D205, D204
// Comments can go anywhere, even above the rules line.
import std;
import os;

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    std::io::fmt_print(Lend.mut $out, "fine{newln}");
    return void;
end fn.main
