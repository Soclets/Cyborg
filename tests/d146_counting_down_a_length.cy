// A length is unsigned, and the step needn't be.
// trace: D146, D142, D140
import std;
import os;

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $xs: std::coll::array<i32>[len: 3 items] = [4, 5, 6];
    for $i in range[from: $xs.len, to: 1, step_by: -1]:
        std::io::fmt_print(Lend.mut $out, "{}{newln}", $xs[pos: $i]);
    end for
    return void;
end fn.main
