// trace: D46
import std;
import os;

fam.handler say_failed:
    say_failed.log:
        case print_failed:
            std::io::fmt_print(Lend.mut $out, "nope{newln}") ?? void;
    end say_failed.log
end fam.handler say_failed

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    std::io::fmt_print(Lend.mut $out, "hi{newln}") otherwise say_failed.*log;
    return void;
end fn.main
