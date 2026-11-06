// trace: D46
import std;

narrowings.handler say_failed:
    say_failed.bail:
        case print_failed:
            return void;
    end say_failed.bail
end narrowings.handler say_failed

fn.main [] -> void:
    std::println("hi") otherwise say_failed.*bail;
    return void;
end fn.main
