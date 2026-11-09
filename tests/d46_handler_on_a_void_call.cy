// A handler narrowing is used at a call as if written there.
// trace: D46
import std;
@config(default);

fam.handler say_failed:
    say_failed.log:
        case print_failed:
            std::println("nope") ?? void;
    end say_failed.log
end fam.handler say_failed

fn.main [] -> void:
    std::println("hi") otherwise say_failed.log;
    return void;
end fn.main
