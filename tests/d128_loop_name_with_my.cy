// trace: D128, D75
import std;
import os;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    for x $i in range[from: 1, to: 2]:
        std::io::fmt_print(Lend.mut $out, "{}{newln}", $i);
        break my::x;
    end for x
    return void;
end fn.main
