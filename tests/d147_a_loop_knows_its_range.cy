// A compiler skips this position's check, and the oracle runs it and it passes, with the same output.
// trace: D147
import std;
import os;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $xs: std::array<i32>[len: 3 items] = [4, 5, 6];
    for $i in range[from: 1, to: $xs.len]:
        std::io::fmt_print(Lend.mut $out, "{}{newln}", $xs[pos: $i]);
    end for
    return void;
end fn.main
