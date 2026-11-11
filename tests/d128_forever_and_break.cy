// trace: D128
import std;
import os;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let mut $n: i32 set_to 0;
    forever:
        change $n set_to $n + 1;
        if $n == 3:
            break;
        end if
    end forever
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $n);
    return void;
end fn.main
