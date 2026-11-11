// Never read, so nothing reads it empty.
// trace: D203
import std;
import os;

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    decl $x: i32;
    std::io::fmt_print(Lend.mut $out, "fine{newln}");
    return void;
end fn.main
