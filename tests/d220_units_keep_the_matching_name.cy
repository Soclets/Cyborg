// A unit's narrowing isn't a function's, so it keeps D150.
// trace: D220, D150
import std;

use std::units::time.seconds as seconds;

fn.main [none] returns void:
    let $t: i32[unit: seconds.span] set_to 5 seconds;
    return void;
end fn.main
