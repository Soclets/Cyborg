// An arm written at the call needs no `*`.
// trace: D128, D45
import std;
@config(default);

fn.main [] -> void:
    forever:
        std::io::fmt_print("once{newln}") otherwise:
            case print_failed:
                break;
        end otherwise
        break;
    end forever
    return void;
end fn.main
