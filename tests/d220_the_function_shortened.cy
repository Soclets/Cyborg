// Shorten the function, and write the narrowing after it.
// trace: D220, D150
@config(default);
import std;

use std::io::fmt_print as fmt_print;

fn.main [] -> void:
    fmt_print.stderr("to stderr{newln}");
    return void;
end fn.main
