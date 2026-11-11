// trace: D126, D11
import std;
import os;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    if 1 below 2:
        std::io::fmt_print(Lend.mut $out, "yes{newln}");
        return void;
    else:
        return void;
    end if
end fn.main
