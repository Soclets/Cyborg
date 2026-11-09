// trace: D112, D93
import std;

use std::time.seconds as seconds;
@config(default);

struct reading:
    time: i64.std::time.seconds;
end struct reading

fn.main [] -> void:
    let $d: i32.seconds = 3600 seconds;
    let $r: reading = reading[time = 3600 std::time.seconds];
    std::println($d as str);
    return void;
end fn.main
