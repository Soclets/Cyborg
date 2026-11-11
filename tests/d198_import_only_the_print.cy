// trace: D198, D199
import std::io::fmt_print;
import os;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let mut $err: os::stderr.unbuffered = os::stderr.unbuffered();
    std::io::fmt_print(Lend.mut $out, "only this{newln}");
    std::io::fmt_print(Lend.mut $err, "and its narrowings{newln}");
    return void;
end fn.main
