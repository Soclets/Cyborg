// trace: D209
import std;
import os;

fn check [$n: i32] returns void fails [negative]:
    if $n below 0:
        fail negative;
    end if
    return void;
end fn check

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    check(-1) otherwise:
        case negative:
            std::io::fmt_print(Lend.mut $out, "negative{newln}");
    end otherwise
    return void;
end fn.main
