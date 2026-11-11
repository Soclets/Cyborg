// trace: D50, D103
import std;
import os;

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $a: str = "hi";
    let $b: str = empty;
    std::io::fmt_print(Lend.mut $out, "{}{}!{newln}", $a, $b);
    return void;
end fn.main
