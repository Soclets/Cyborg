// trace: D133, D26
import std;
import os;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let $xs: std::list<i32>[len.max: 8 items] set_to [10, 20];
    std::io::fmt_print(Lend.mut $out, "before{newln}");
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $xs[pos: 3]);
    std::io::fmt_print(Lend.mut $out, "after{newln}");
    return void;
end fn.main
