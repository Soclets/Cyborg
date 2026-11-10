// A unit's narrowing isn't a function's, so it keeps D150.
// trace: D220, D150
@config(default);
import std;

use std::units::time.seconds as seconds;

fn.main [] -> void:
    let $t: i32.seconds.span = 5 seconds;
    return void;
end fn.main
