// `std` is a kit and `time` the family, so this is still one level.
// trace: D161, D150
import std;
import os;

use std::units::time.seconds as seconds;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let $a: i32[unit: seconds.span] set_to 3600 seconds;
    std::io::fmt_print(Lend.mut $out, "ok{newln}");
    return void;
end fn.main
