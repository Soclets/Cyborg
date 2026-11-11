// trace: D199, D150
import std;
import os;
use std::units::time.seconds as seconds;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $a: i32[unit: seconds.span] = 60 seconds;
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $a);
    return void;
end fn.main
