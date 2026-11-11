// `.stdout` is gone too.
// trace: D272, D273
import std;
import os;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    std::io::fmt_print.stdout(Lend.mut $out, "hi{newln}");
    return void;
end fn.main
