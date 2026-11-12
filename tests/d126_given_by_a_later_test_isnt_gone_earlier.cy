// The second test hasn't run when the first block runs.
// trace: D126, D2
import std;
import os;

fn eat [$s: Given str[len.max: 32 bytes]] -> bool:
    return 1 below 2;
end fn eat

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $s: str[len.max: 32 bytes] = "a";
    if 1 below 2:
        std::io::fmt_print(Lend.mut $out, "{}{newln}", $s);
    else if eat(Give $s):
        return void;
    end if
    return void;
end fn.main
