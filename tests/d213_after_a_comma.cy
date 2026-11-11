// A comma is followed by one space.
// trace: D213
import std;
import os;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    std::io::fmt_print(Lend.mut $out, "{} {}{newln}", 1,2);
    return void;
end fn.main
