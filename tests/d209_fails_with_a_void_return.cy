// trace: D209
import std;
import os;

fn check [$n: i32] -> void fails [negative]:
    if $n < 0:
        fail negative;
    end if
    return void;
end fn check

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    check(-1) otherwise:
        case negative:
            std::io::fmt_print(Lend.mut $out, "negative{newln}");
    end otherwise
    return void;
end fn.main
