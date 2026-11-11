// trace: D132
import std;
import os;

fn read [none] returns i32 fails [none]:
    fail none;
end fn read

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $x: i32 = 5;
    std::io::fmt_print(Lend.mut $out, "{}{newln}", (read() ?? -1));
    std::io::fmt_print(Lend.mut $out, "{}{newln}", (read() ?? $x));
    std::io::fmt_print(Lend.mut $out, "{}{newln}", (read() ?? ($x + 1)));
    return void;
end fn.main
