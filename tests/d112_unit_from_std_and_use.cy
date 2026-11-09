// trace: D112, D93
import std;

use std::units::time.seconds as seconds;
@config(default);

struct reading:
    time: i64.std::units::time.seconds;
end struct reading

fn.main [] -> void:
    let $d: i32.seconds = 3600 seconds;
    let $r: reading = reading[time = 3600 std::units::time.seconds];
    std::io::fmt_print("{}{newln}", $d);
    return void;
end fn.main
