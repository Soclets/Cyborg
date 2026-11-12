// trace: D101
import std;
import os;

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $xs: std::coll::array<i32>[len: 3 items] = [1, 2, 3];
    if $xs.len at_least 3 items:
        std::io::fmt_print(Lend.mut $out, "three{newln}");
    end if
    return void;
end fn.main
