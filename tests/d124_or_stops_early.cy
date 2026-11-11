// trace: D124
import std;
import os;

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $n: i32 = 0;
    if $n == 0 or 10 div.trunc $n above 1:
        std::io::fmt_print(Lend.mut $out, "safe{newln}");
    end if
    return void;
end fn.main
