// trace: D146, D142
import std;
import os;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $xs: std::array<i32>[len: 4 items] = [4, 5, 6, 7];
    for $i in range[from: 1, to: $xs.len, step_by: 2 items]:
        std::io::fmt_print(Lend.mut $out, "{}{newln}", $xs[pos: $i]);
    end for
    return void;
end fn.main
