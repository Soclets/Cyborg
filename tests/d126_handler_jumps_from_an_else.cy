// A `return` in any block of a chain makes a handler one that can jump.
// trace: D126, D46
import std;

narrowings.handler say_failed:
    say_failed.maybe:
        case print_failed:
            if 1 < 2:
                std::println("x") ?? void;
            else:
                return void;
            end if
    end say_failed.maybe
end narrowings.handler say_failed

fn.main [] -> void:
    std::println("hi") otherwise say_failed.maybe;
    return void;
end fn.main
