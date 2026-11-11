// trace: D117
import std;
import os;

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $x: i32 = 3;
    if 1 < $x < 5:
        std::io::fmt_print(Lend.mut $out, "between{newln}");
    end if
    if 1 < $x < 2:
        std::io::fmt_print(Lend.mut $out, "not reached{newln}");
    end if
    if 1 < 2 at_most 2 == 2 != 3:
        std::io::fmt_print(Lend.mut $out, "any mix{newln}");
    end if
    return void;
end fn.main
