// trace: D203
import std;
import os;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    decl mut $x: i32;
    assign $x = 1;
    change $x = $x + 41;
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $x);
    return void;
end fn.main
