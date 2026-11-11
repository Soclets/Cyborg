// A buffered writer is `must_Give`, so a program that never gives it to `os::done` doesn't
// compile.
// trace: D272, D273, D129
import std;
import os;

fn.main [none] returns void:
    let mut $out: os::stdout.buffered = os::stdout.buffered(size: 1024 bytes);
    std::io::fmt_print(Lend.mut $out, "lost?{newln}");
    return void;
end fn.main
