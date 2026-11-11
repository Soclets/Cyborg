// `with` adds features, which aren't designed.
// trace: D228
import std;
import os;

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    std::io::fmt_print(Lend.mut $out, "hi{newln}");
    return void;
end fn.main
