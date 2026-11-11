// A `return` in any block of a chain makes a handler one that can jump.
// trace: D126, D46
import std;
import os;

fam.handler say_failed:
    say_failed.maybe:
        case print_failed:
            if 1 < 2:
                std::io::fmt_print(Lend.mut $out, "x{newln}") ?? void;
            else:
                return void;
            end if
    end say_failed.maybe
end fam.handler say_failed

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    std::io::fmt_print(Lend.mut $out, "hi{newln}") otherwise say_failed.maybe;
    return void;
end fn.main
