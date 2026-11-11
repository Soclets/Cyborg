// trace: D133, D20
import std;
import os;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let mut $a: std::array<i32[min: 0, max: 9]>[len: 1 items] set_to [1];
    let $big: i32 set_to 10;
    std::io::fmt_print(Lend.mut $out, "before{newln}");
    change $a[pos: 1] set_to $big;
    std::io::fmt_print(Lend.mut $out, "after{newln}");
    return void;
end fn.main
