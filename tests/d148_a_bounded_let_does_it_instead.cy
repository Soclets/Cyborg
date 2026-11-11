// trace: D148, D20
import std;
import os;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $start: i32[min: 1, max: 3] = 2;
    for $i in range[from: $start, to: 3]:
        std::io::fmt_print(Lend.mut $out, "{}{newln}", $i);
    end for
    return void;
end fn.main
