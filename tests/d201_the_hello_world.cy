// The standard's hello world, with its rules line after the imports and use.
// trace: D201, D197
import std;
use std::io::fmt_print as fmt_print;
@config(default);

fn.main [] -> void:
    fmt_print("Hello, World!{newln}");
    return void;
end fn.main
