// trace: D203
import std;
import os;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    decl $x: i32;
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $x);
    return void;
end fn.main
