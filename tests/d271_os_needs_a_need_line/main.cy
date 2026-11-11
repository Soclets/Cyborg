// A program that uses `os` says `need os` in its manifest.
// trace: D271, D183
import std;
import os;

fn.main [none] -> void:
    let mut $out: os::stdout.buffered = os::stdout.buffered(size: 1024 bytes);
    std::io::fmt_print(Lend.mut $out, "hi{newln}");
    os::done(Give $out);
    return void;
end fn.main
