// Nothing here says which float this is.
// trace: D161
import std;
import os;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    std::io::fmt_print(Lend.mut $out, "{}{newln}", ~0.1);
    return void;
end fn.main
