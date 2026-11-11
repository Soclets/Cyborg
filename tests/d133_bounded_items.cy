// trace: D133, D20
import std;
import os;

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let mut $a: std::array<i32[min: 0, max: 9]>[len: 1 items] = [1];
    let $big: i32 = 10;
    std::io::fmt_print(Lend.mut $out, "before{newln}");
    change $a[pos: 1] = $big;
    std::io::fmt_print(Lend.mut $out, "after{newln}");
    return void;
end fn.main
