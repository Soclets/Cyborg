// trace: D188, D155
import pkg std;
import os;

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    std::io::fmt_print(Lend.mut $out, "both halves{newln}");
    return void;
end fn.main
