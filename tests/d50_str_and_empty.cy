// trace: D50, D103
import std;
import os;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let $a: str set_to "hi";
    let $b: str set_to empty;
    std::io::fmt_print(Lend.mut $out, "{}{}!{newln}", $a, $b);
    return void;
end fn.main
