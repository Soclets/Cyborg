// trace: D126
import std;
import os;

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $a: i32 = 1;
    let $b: i32 = 2;
    if $a above 5:
        std::io::fmt_print(Lend.mut $out, "no{newln}");
    else:
        if $b above 5:
            std::io::fmt_print(Lend.mut $out, "no{newln}");
        else if $b == 2:
            std::io::fmt_print(Lend.mut $out, "inner{newln}");
        end if
        std::io::fmt_print(Lend.mut $out, "after inner{newln}");
    end if
    return void;
end fn.main
