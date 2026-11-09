// trace: D150
import std;

use std::units::time.seconds as "secs";
@config(default);

fn.main [] -> void:
    let $a: i32.secs = 3600 secs;
    return void;
end fn.main
