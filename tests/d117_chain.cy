// trace: D117
import std;
import os;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $x: i32 = 3;
    if 1 below $x below 5:
        std::io::fmt_print(Lend.mut $out, "between{newln}");
    end if
    if 1 below $x below 2:
        std::io::fmt_print(Lend.mut $out, "not reached{newln}");
    end if
    if 1 below 2 at_most 2 == 2 != 3:
        std::io::fmt_print(Lend.mut $out, "any mix{newln}");
    end if
    return void;
end fn.main
