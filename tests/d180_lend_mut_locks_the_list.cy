// trace: D180, D2
import std;
import os;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let mut $xs: std::array<i32>[len: 3 items] = [1, 2, 3];
    for $n in Lend.mut $xs:
        std::io::fmt_print(Lend.mut $out, "{}{newln}", $xs.len);
    end for
    return void;
end fn.main
