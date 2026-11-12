// `for $x in Lend $xs:` reads each item in place, next to `Lend.mut` and `Give`.
// trace: D393, D180
import std;
import os;

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $xs: std::coll::array<i32>[len: 3 items] = [1, 2, 3];
    let mut $sum: i32 = 0;
    for $n in Lend $xs:
        change $sum = $sum + $n;
    end for
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $sum);
    return void;
end fn.main
