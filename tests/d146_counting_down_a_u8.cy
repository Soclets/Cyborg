// A `u8` can't hold -1, and a step isn't stored, so it doesn't have to.
// trace: D146
import std;
import os;

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $from: u8 = 3;
    let $to: u8 = 0;
    for $i in range[from: $from, to: $to, step_by: -1]:
        std::io::fmt_print(Lend.mut $out, "{}{newln}", $i);
    end for
    return void;
end fn.main
