// trace: D150
import std;

use std::io::fmt_print as fmt_print;
@config(default);

fn.main [] -> void:
    fmt_print("hi{newln}");
    return void;
end fn.main
