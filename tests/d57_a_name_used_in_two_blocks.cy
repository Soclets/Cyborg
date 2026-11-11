// A block's names end with it, so the next block can use the name again.
// trace: D57, D33
import std;
import os;

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    if 1 < 2:
        let $a: i32 = 1;
        std::io::fmt_print(Lend.mut $out, "{}{newln}", $a);
    end if
    if 1 < 2:
        let $a: i32 = 2;
        std::io::fmt_print(Lend.mut $out, "{}{newln}", $a);
    end if
    return void;
end fn.main
