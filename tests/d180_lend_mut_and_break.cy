// trace: D180, D128
import std;
import os;

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let mut $xs: std::coll::array<i32>[len: 3 items] = [1, 2, 3];
    for $n in Lend.mut $xs:
        if $n == 2:
            break;
        end if
        change $n = 0;
    end for
    for $n in Lend $xs:
        std::io::fmt_print(Lend.mut $out, "{}{newln}", $n);
    end for
    return void;
end fn.main
