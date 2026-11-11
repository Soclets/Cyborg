// trace: D128
import std;
import os;

fn.pure again [none] -> bool:
    return false;
end fn.pure again

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    while again():
        return void;
    end while
    std::io::fmt_print(Lend.mut $out, "done{newln}");
    return void;
end fn.main
