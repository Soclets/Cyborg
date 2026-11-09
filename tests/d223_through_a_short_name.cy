// The order holds through a short name too, and the fix keeps the short name.
// trace: D223, D220
@config(default);
import std;

use std::io::fmt_print as fmt_print;

fn.main [] -> void:
    fmt_print.stderr.stored("{}{newln}", 0.5);
    return void;
end fn.main
