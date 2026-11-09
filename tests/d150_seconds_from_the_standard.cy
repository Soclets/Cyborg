// trace: D150
import std;

use std::units::time.seconds as seconds;
@config(default);

fn.main [] -> void:
    let $a: i32.seconds = 3600 seconds;
    return void;
end fn.main
