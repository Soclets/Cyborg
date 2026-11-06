// An arm written at the call needs no `*`.
// trace: D128, D45
import std;

fn.main [] -> void:
    forever:
        std::println("once") otherwise:
            case print_failed:
                break;
        end otherwise
        break;
    end forever
    return void;
end fn.main
