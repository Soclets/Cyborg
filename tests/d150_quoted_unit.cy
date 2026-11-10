// trace: D150
@config(default);
import std;

use std::units::time.seconds as "secs";

fn.main [] -> void:
    let $a: i32.secs.span = 3600 secs;
    return void;
end fn.main
