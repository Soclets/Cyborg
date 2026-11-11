// trace: D150
import std;

use std::units::time.seconds as "secs";

fn.main [none] returns void:
    let $a: i32[unit: secs.span] set_to 3600 secs;
    return void;
end fn.main
