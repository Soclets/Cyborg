// trace: D199, D150
import std;
use std::units::time.seconds as seconds;
@config(default);

fn.main [] -> void:
    let $a: i32.seconds = 60 seconds;
    std::io::fmt_print("{}{newln}", $a);
    return void;
end fn.main
