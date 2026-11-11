// trace: D132
import std;
import os;

fn read [none] returns i32 fails [none]:
    fail none;
end fn read

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $a: i32 = (read()) ?? 4;
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $a);
    return void;
end fn.main
