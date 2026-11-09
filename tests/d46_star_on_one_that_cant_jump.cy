// trace: D46
@config(default);
import std;

fam.handler say_failed:
    say_failed.log:
        case print_failed:
            std::io::fmt_print("nope{newln}") ?? void;
    end say_failed.log
end fam.handler say_failed

fn.main [] -> void:
    std::io::fmt_print("hi{newln}") otherwise say_failed.*log;
    return void;
end fn.main
