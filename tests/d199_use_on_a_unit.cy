// trace: D199, D150
@config(default);
import std;
use std::units::time.seconds as seconds;

fn.main [] -> void:
    let $a: i32.seconds.span = 60 seconds;
    std::io::fmt_print("{}{newln}", $a);
    return void;
end fn.main
