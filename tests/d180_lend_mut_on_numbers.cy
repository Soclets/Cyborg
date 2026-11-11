// The word is on the list, so a list of numbers takes one too, and each number changes in place.
// trace: D180, D123
import std;
import os;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let mut $xs: std::array<i32>[len: 3 items] set_to [1, 2, 3];
    for $n in Lend.mut $xs:
        change $n set_to $n x 10;
    end for
    for $n in $xs:
        std::io::fmt_print(Lend.mut $out, "{}{newln}", $n);
    end for
    return void;
end fn.main
