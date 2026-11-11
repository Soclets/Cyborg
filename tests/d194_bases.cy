// trace: D194
import std;
import os;

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    std::io::fmt_print(Lend.mut $out, "{base: bin} {base: oct} {base: hex} {base: hex.upper}{newln}", 10, 10, 255, 255);
    return void;
end fn.main
