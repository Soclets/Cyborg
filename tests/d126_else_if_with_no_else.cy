// trace: D126
import std;
import os;

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $n: i32 = 0;
    if $n above 0:
        std::io::fmt_print(Lend.mut $out, "positive{newln}");
    else if $n below 0:
        std::io::fmt_print(Lend.mut $out, "negative{newln}");
    end if
    std::io::fmt_print(Lend.mut $out, "carried on{newln}");
    return void;
end fn.main
