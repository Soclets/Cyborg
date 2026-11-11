// trace: D126, D11
import std;
import os;

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    if 1 < 2:
        std::io::fmt_print(Lend.mut $out, "yes{newln}");
        return void;
    else:
        return void;
    end if
end fn.main
