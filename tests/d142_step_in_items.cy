// trace: D142, D128
import std;
import os;

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $xs: std::coll::array<i32>[len: 5 items] = [1, 2, 3, 4, 5];
    for $i in range[from: 1, to: $xs.len, step_by: 2 items]:
        std::io::fmt_print(Lend.mut $out, "{}{newln}", $i);
    end for
    return void;
end fn.main
