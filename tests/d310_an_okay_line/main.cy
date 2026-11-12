// The `okay` line is refused, with the `dialect` line and its version as the fix.
// trace: D310
import std;
import os;

fn.main [none] -> void:
    let mut $out: os::stdout.buffered = os::stdout.buffered(room: 1024 bytes);
    std::io::fmt_print(Lend.mut $out, "hi{newln}");
    os::done(Give $out);
    return void;
end fn.main
