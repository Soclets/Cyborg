// A step isn't stored, so a `u8` range can take 300, and jumps past its end.
// trace: D146
import std;
import os;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $from: u8 = 1;
    let $to: u8 = 5;
    for $i in range[from: $from, to: $to, step_by: 300]:
        std::io::fmt_print(Lend.mut $out, "{}{newln}", $i);
    end for
    return void;
end fn.main
