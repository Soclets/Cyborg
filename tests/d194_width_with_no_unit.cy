// trace: D194
import std;
import os;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    std::io::fmt_print(Lend.mut $out, "{align: left, width: 8}{newln}", "a");
    return void;
end fn.main
