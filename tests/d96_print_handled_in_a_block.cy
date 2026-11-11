// trace: D45, D96
import std;
import os;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    std::io::fmt_print(Lend.mut $out, "hi{newln}") otherwise:
        case print_failed:
            return void;
    end otherwise
    return void;
end fn.main
