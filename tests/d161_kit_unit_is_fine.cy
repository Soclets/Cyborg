// `std` is a kit and `time` the family, so this is still one level.
// trace: D161, D150
import std;

use std::time.seconds as seconds;
@config(default);

fn.main [] -> void:
    let $a: i32.seconds = 3600 seconds;
    std::io::fmt_print("ok{newln}");
    return void;
end fn.main
