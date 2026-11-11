// trace: D150
import std;

use std::units::time.seconds as "secs";

fn.main [none] -> void:
    let $a: i32[unit: secs.span] = 3600 secs;
    return void;
end fn.main
