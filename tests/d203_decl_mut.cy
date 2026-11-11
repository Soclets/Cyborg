// trace: D203
import std;
import os;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    decl mut $x: i32;
    assign $x set_to 1;
    change $x set_to $x + 41;
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $x);
    return void;
end fn.main
