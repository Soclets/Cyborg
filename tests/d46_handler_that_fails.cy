// `fail` leaves the function like `return` does, though D46 lists only `return`, `break` and
// `continue` as what needs a `*`. Reported to the design session.
// trace: D46, D52
@config(default);
import std;

fam.handler say_failed:
    say_failed.up:
        case print_failed:
            fail lost;
    end say_failed.up
end fam.handler say_failed

fn say [] -> void fails [lost]:
    std::io::fmt_print("hi{newln}") otherwise say_failed.up;
    return void;
end fn say

fn.main [] -> void:
    say();
    return void;
end fn.main
