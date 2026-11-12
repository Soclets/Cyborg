// trace: D125, D2
import std;
import os;

struct tally:
    total: i32;
end struct tally

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $xs: std::coll::array<i32>[len: 2 items] = [1, 2];
    let mut $t: tally = tally[total = 0];
    for $x in $xs:
        change $t.total = $t.total + $x;
    end for
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $t.total);
    return void;
end fn.main
