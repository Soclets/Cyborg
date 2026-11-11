// trace: D116, D44
import std;
import os;

fn read [none] returns i32 fails [none]:
    fail none;
end fn read

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let $a: i32 set_to 1;
    let $b: i32 set_to $a + (read() ?? 5);
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $b);
    return void;
end fn.main
