// `fail` leaves the function like `return` does, though D46 lists only `return`, `break` and
// `continue` as what needs a `*`. Reported to the design session.
// trace: D46, D52
import std;
import os;

fam.handler say_failed:
    say_failed.up:
        case print_failed:
            fail lost;
    end say_failed.up
end fam.handler say_failed

fn say [none] returns void fails [lost]:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    std::io::fmt_print(Lend.mut $out, "hi{newln}") otherwise say_failed.up;
    return void;
end fn say

fn.main [none] returns void:
    say();
    return void;
end fn.main
