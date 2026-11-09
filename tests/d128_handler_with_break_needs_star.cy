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
    forever:
        std::println("hi") otherwise say_failed.stop;
        break;
    end forever
    return void;
end fn.main
