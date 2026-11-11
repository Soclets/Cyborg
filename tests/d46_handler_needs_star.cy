// trace: D46
import std;
import os;

fam.handler say_failed:
    say_failed.bail:
        case print_failed:
            return void;
    end say_failed.bail
end fam.handler say_failed

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    std::io::fmt_print(Lend.mut $out, "hi{newln}") otherwise say_failed.bail;
    return void;
end fn.main
