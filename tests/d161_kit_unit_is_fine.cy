// `std` is a kit and `time` the family, so this is still one level.
// trace: D161, D150
import std;
import os;

use std::units::time.seconds as seconds;

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $a: i32[unit: seconds.span] = 3600 seconds;
    std::io::fmt_print(Lend.mut $out, "ok{newln}");
    return void;
end fn.main
