// trace: D142, D128
import std;
import os;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $xs: std::array<i32>[len: 3 items] = [4, 5, 6];
    for $i in range[from: 1, to: $xs.len]:
        std::io::fmt_print(Lend.mut $out, "{} holds {}{newln}", $i, $xs[pos: $i]);
    end for
    for $i in range[from: 1 items, before: $xs.len]:
        std::io::fmt_print(Lend.mut $out, "{}{newln}", $i);
    end for
    for $i in range[from: 1 items, to: 2 items]:
        let $kept: i32[unit: items] = $i;
        std::io::fmt_print(Lend.mut $out, "{}{newln}", $kept);
    end for
    return void;
end fn.main
