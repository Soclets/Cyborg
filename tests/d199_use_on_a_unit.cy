// trace: D199, D150
import std;
use std::units::time.seconds as seconds;

fn.main [none] -> void:
    let $a: i32[unit: seconds.span] = 60 seconds;
    std::io::fmt_print("{}{newln}", $a);
    return void;
end fn.main
