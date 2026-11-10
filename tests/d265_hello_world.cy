// The standard's hello world, straight to the point, with its rules in the manifest.
// trace: D265, D267, D197
import std;
use std::io::fmt_print as fmt_print;

fn.main [none] -> void:
    fmt_print("Hello, World!{newln}");
    return void;
end fn.main
