// trace: D45, D96
import std;
@config(default);

fn.main [] -> void:
    std::println("hi") otherwise:
        case print_failed:
            return void;
    end otherwise
    return void;
end fn.main
