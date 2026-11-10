// trace: D150
@config(default);
import std;

use std::units::time.seconds as seconds;

fn.main [] -> void:
    let $a: i32.seconds.span = 3600 seconds;
    return void;
end fn.main
