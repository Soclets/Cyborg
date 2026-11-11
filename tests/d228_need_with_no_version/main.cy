// A `need` line with no version is refused.
// trace: D228, D231
import std;
import os;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    std::io::fmt_print(Lend.mut $out, "hi{newln}");
    return void;
end fn.main
