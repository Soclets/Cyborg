// The word is on the list, so a list of numbers takes one too, and each number changes in place.
// trace: D180, D123
import std;
import os;

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let mut $xs: std::coll::array<i32>[len: 3 items] = [1, 2, 3];
    for $n in Lend.mut $xs:
        change $n = $n x 10;
    end for
    for $n in $xs:
        std::io::fmt_print(Lend.mut $out, "{}{newln}", $n);
    end for
    return void;
end fn.main
