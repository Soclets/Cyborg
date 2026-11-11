// trace: D124
import std;
import os;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $n: i32 = 0;
    if $n != 0 and 10 div.trunc $n above 1:
        std::io::fmt_print(Lend.mut $out, "not reached{newln}");
    end if
    std::io::fmt_print(Lend.mut $out, "safe{newln}");
    return void;
end fn.main
