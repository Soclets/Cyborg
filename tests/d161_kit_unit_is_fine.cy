// `std` is a kit and `time` the family, so this is still one level.
// trace: D161, D150
import std;

use std::time.seconds as seconds;

fn.main [] -> void:
    let $a: i32.seconds = 3600 seconds;
    std::println("ok");
    return void;
end fn.main
