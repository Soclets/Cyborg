// trace: D128
import std;
import os;

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let mut $n: i32 = 0;
    forever:
        change $n = $n + 1;
        if $n == 3:
            break;
        end if
    end forever
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $n);
    return void;
end fn.main
