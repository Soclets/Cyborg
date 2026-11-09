// trace: D125, D2
@config(default);
import std;

struct tally:
    total: i32;
end struct tally

fn.main [] -> void:
    let $xs: std::array<i32>[len: 2 items] = [1, 2];
    let mut $t: tally = tally[total = 0];
    for $x in $xs:
        change $t.total = $t.total + $x;
    end for
    std::io::fmt_print("{}{newln}", $t.total);
    return void;
end fn.main
