// trace: D116, D44
import std;
import os;

fn read [none] -> i32 fails [none]:
    fail none;
end fn read

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $a: i32 = 1;
    let $b: i32 = $a + (read() ?? 5);
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $b);
    return void;
end fn.main
