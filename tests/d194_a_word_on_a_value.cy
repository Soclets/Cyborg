// trace: D194
import std;
import os;

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $s: str = "a";
    std::io::fmt_print(Lend.mut $out, "{}{newln}", Lend $s);
    return void;
end fn.main
