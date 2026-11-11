// trace: D26
import std;
import os;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $z: i32 = 0;
    std::io::fmt_print(Lend.mut $out, "before{newln}");
    let $a: i32 = 1 div.trunc $z;
    std::io::fmt_print(Lend.mut $out, "after{newln}");
    return void;
end fn.main
