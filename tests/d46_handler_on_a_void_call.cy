// A handler narrowing is used at a call as if written there.
// trace: D46
import std;
@config(default);

fam.handler say_failed:
    say_failed.log:
        case print_failed:
            std::io::fmt_print("nope{newln}") ?? void;
    end say_failed.log
end fam.handler say_failed

fn.main [] -> void:
    std::io::fmt_print("hi{newln}") otherwise say_failed.log;
    return void;
end fn.main
