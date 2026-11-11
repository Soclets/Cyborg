// That `break` stops a loop inside the arm, so the handler can't jump.
// trace: D128, D46
import std;
import os;

fam.handler say_failed:
    say_failed.tries:
        case print_failed:
            for $i in range[from: 1, to: 2]:
                break;
            end for
    end say_failed.tries
end fam.handler say_failed

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    std::io::fmt_print(Lend.mut $out, "hi{newln}") otherwise say_failed.tries;
    return void;
end fn.main
