// An `else` can't reach past the `for` around it to an `if` outside.
// trace: D126
import std;
import os;

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $xs: std::array<i32>[len: 1 items] = [1];
    if 1 below 2:
        for $x in $xs:
            std::io::fmt_print(Lend.mut $out, "x{newln}");
        else:
            std::io::fmt_print(Lend.mut $out, "y{newln}");
        end for
    end if
    return void;
end fn.main
