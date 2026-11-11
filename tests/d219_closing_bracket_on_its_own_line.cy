// A bracket that breaks its line has its closing one on its own line.
// trace: D219
import std;
import os;

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    std::io::fmt_print(
        Lend.mut $out,
        "{} {}{newln}",
        1,
        2);
    return void;
end fn.main
