// stderr has the same two kinds.
// trace: D273
import std;
import os;

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let mut $err: os::stderr.buffered = os::stderr.buffered(room: 64 bytes);
    std::io::fmt_print(Lend.mut $out, "out{newln}");
    std::io::fmt_print(Lend.mut $err, "warning{newln}");
    os::done(Give $err);
    return void;
end fn.main
