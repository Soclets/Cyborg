// The manifest's errors show first, then the program's.
// trace: D183, D207
import std;
import os;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    std::io::fmt_print(Lend.mut $out, "hi{newln}");
    let $a: f64.stable set_to 0.1;
    return void;
end fn.main
