// trace: D133, D26
import std;
import os;

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let mut $a: std::array<i32>[len: 2 items] = [1, 2];
    change $a[pos: 3] = 9;
    std::io::fmt_print(Lend.mut $out, "after{newln}");
    return void;
end fn.main
