// Used as if written there, so there's no loop for it to stop.
// trace: D128, D46
import std;
import os;

fam.handler say_failed:
    say_failed.stop:
        case print_failed:
            break;
    end say_failed.stop
end fam.handler say_failed

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    std::io::fmt_print(Lend.mut $out, "hi{newln}") otherwise say_failed.*stop;
    return void;
end fn.main
