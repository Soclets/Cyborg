// trace: D194
import std;
import os;

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    std::io::fmt_print(Lend.mut $out, "{1} {}{newln}", 1, 2);
    return void;
end fn.main
