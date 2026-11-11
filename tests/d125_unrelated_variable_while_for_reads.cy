// trace: D125, D2
import std;
import os;

struct tally:
    total: i32;
end struct tally

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let $xs: std::array<i32>[len: 2 items] set_to [1, 2];
    let mut $t: tally set_to tally[total set_to 0];
    for $x in $xs:
        change $t.total set_to $t.total + $x;
    end for
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $t.total);
    return void;
end fn.main
