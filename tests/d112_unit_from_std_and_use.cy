// trace: D112, D93
import std;

use std::units::time.seconds as seconds;

struct reading:
    time: i64[unit: std::units::time.seconds.span];
end struct reading

fn.main [none] -> void:
    let $d: i32[unit: seconds.span] = 3600 seconds;
    let $r: reading = reading[time = 3600 std::units::time.seconds];
    std::io::fmt_print("{}{newln}", $d);
    return void;
end fn.main
