// Used as if written there, so there's no loop for it to stop.
// trace: D128, D46
import std;
@config(default);

fam.handler say_failed:
    say_failed.stop:
        case print_failed:
            break;
    end say_failed.stop
end fam.handler say_failed

fn.main [] -> void:
    std::io::fmt_print("hi{newln}") otherwise say_failed.*stop;
    return void;
end fn.main
