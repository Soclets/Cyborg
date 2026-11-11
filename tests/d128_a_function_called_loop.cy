// `loop` and `continue` aren't words of Cyborg, so they're free as names.
// trace: D128, D75
import std;
import os;

fn loop [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    std::io::fmt_print(Lend.mut $out, "called{newln}");
    return void;
end fn loop

fn.main [none] -> void:
    loop();
    return void;
end fn.main
