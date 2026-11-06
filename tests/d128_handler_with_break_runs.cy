// trace: D128, D46
import std;

narrowings.handler say_failed:
    say_failed.stop:
        case print_failed:
            break;
    end say_failed.stop
end narrowings.handler say_failed

fn.main [] -> void:
    forever:
        std::println("hi") otherwise say_failed.*stop;
        break;
    end forever
    return void;
end fn.main
