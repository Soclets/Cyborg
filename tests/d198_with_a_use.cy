// trace: D198, D93
@config(default);
import std::io::fmt_print;
use std::io::fmt_print as fmt_print;

fn.main [] -> void:
    fmt_print("hi{newln}");
    return void;
end fn.main
