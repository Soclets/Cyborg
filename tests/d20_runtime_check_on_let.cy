// trace: D20
import std;
import os;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $n: i32 = 150;
    std::io::fmt_print(Lend.mut $out, "before{newln}");
    let $b: i32[min: 0, max: 100] = $n;
    std::io::fmt_print(Lend.mut $out, "after{newln}");
    return void;
end fn.main
