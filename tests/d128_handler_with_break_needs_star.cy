// trace: D128, D46
import std;

fam.handler say_failed:
    say_failed.stop:
        case print_failed:
            break;
    end say_failed.stop
end fam.handler say_failed

fn.main [none] -> void:
    forever:
        std::io::fmt_print("hi{newln}") otherwise say_failed.stop;
        break;
    end forever
    return void;
end fn.main
