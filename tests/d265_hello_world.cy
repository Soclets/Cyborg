// The standard's hello world, straight to the point, with its rules in the manifest and its
// output through a buffered writer.
// trace: D265, D267, D197, D272, D273
import std;
import os;
use std::io::fmt_print as fmt_print;

fn.main [none] -> void:
    let mut $out: os::stdout.buffered = os::stdout.buffered(room: 1024 bytes);
    fmt_print(Lend.mut $out, "Hello, World!{newln}");
    os::done(Give $out);
    return void;
end fn.main
