// A block opener stays on one line.
// trace: D219
import std;
import os;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $a: i32 = 1;
    if
        $a below 3:
        std::io::fmt_print(Lend.mut $out, "small{newln}");
    end if
    return void;
end fn.main
