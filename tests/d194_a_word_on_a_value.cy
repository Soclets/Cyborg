// trace: D194
import std;
import os;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let $s: str set_to "a";
    std::io::fmt_print(Lend.mut $out, "{}{newln}", Lend $s);
    return void;
end fn.main
