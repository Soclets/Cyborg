// A short name for the print keeps its narrowings.
// trace: D199, D197, D150
import std;

use std::io::fmt_print as fmt_print;

fn.main [none] -> void:
    let $x: f64.stable = 0.5;
    fmt_print.stored("{}{newln}", $x);
    return void;
end fn.main
