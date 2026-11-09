// `std` is a kit and `time` the family, so this is still one level.
// trace: D161, D150
@config(default);
import std;

use std::units::time.seconds as seconds;

fn.main [] -> void:
    let $a: i32.seconds = 3600 seconds;
    std::io::fmt_print("ok{newln}");
    return void;
end fn.main
