// The standard's hello world, with its rules line first.
// trace: D201, D204, D197
@config(default);
import std;
use std::io::fmt_print as fmt_print;

fn.main [] -> void:
    fmt_print("Hello, World!{newln}");
    return void;
end fn.main
