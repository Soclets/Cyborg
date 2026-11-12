// Hello world is a source file plus a manifest.
// trace: D183, D228, D229, D271
import std;
import os;

fn.main [none] -> void:
    let mut $out: os::stdout.buffered = os::stdout.buffered(room: 1024 bytes);
    std::io::fmt_print(Lend.mut $out, "hi{newln}");
    os::done(Give $out);
    return void;
end fn.main
