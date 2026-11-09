// The rules come first, then the imports and `use` lines.
// trace: D204, D201
@config(default);
import std;
use std::io::fmt_print as fmt_print;

fn.main [] -> void:
    fmt_print("rules first{newln}");
    return void;
end fn.main
