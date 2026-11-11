// trace: D187, D155
import std;
import @std;
import os;

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    std::io::fmt_print(Lend.mut $out, "both halves{newln}");
    return void;
end fn.main
