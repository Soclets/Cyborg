// trace: D128
import std;
import os;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    for $a in range[from: 1, to: 2]:
        for $b in range[from: 1, to: 3]:
            if $b == 2:
                break;
            end if
            std::io::fmt_print(Lend.mut $out, "{} {}{newln}", $a, $b);
        end for
    end for
    return void;
end fn.main
