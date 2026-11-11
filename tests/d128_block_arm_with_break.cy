// An arm written at the call needs no `*`.
// trace: D128, D45
import std;
import os;

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    forever:
        std::io::fmt_print(Lend.mut $out, "once{newln}") otherwise:
            case print_failed:
                break;
        end otherwise
        break;
    end forever
    return void;
end fn.main
