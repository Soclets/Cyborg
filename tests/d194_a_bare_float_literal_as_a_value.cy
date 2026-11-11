// A bare float literal as a value says nothing about which float it is.
// trace: D194, D59
import std;
import os;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    std::io::fmt_print(Lend.mut $out, "{form: science, places: 2}{newln}", 12.5);
    return void;
end fn.main
