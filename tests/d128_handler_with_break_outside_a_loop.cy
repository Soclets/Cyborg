// Used as if written there, so there's no loop for it to stop.
// trace: D128, D46
import std;

narrowings.handler say_failed:
    say_failed.stop:
        case print_failed:
            break;
    end say_failed.stop
end narrowings.handler say_failed

fn.main [] -> void:
    std::println("hi") otherwise say_failed.*stop;
    return void;
end fn.main
