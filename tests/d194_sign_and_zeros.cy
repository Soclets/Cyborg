// trace: D194
import std;
import os;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $n: i32 = -42;
    std::io::fmt_print(Lend.mut $out, "{sign} {sign} {zeros, width: 6 cols} {sign, zeros, width: 6 cols}{newln}", 7, $n, $n, 7);
    return void;
end fn.main
