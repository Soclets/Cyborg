// `.stderr` is gone, since the writer says where output goes.
// trace: D272, D273
import std;
import os;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    std::io::fmt_print.stderr(Lend.mut $out, "oops{newln}");
    return void;
end fn.main
