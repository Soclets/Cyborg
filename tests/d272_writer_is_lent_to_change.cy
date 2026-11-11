// A print changes its writer, so it's lent with `Lend.mut`.
// trace: D272, D3
import std;
import os;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    std::io::fmt_print($out, "hi{newln}");
    return void;
end fn.main
