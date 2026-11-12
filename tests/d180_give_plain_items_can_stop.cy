// Numbers carry no duty, so stopping early leaves nothing owed.
// trace: D180, D128
import std;
import os;

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $xs: std::coll::list<i32>[len.max: inf items] = [1, 2, 3];
    for $n in Give $xs:
        if $n == 2:
            break;
        end if
        std::io::fmt_print(Lend.mut $out, "{}{newln}", $n);
    end for
    return void;
end fn.main
