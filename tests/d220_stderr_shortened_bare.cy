// The same goes for `.stderr`.
// trace: D220, D150
@config(default);
import std;

use std::io::fmt_print.stderr as stderr;

fn.main [] -> void:
    stderr("x{newln}");
    return void;
end fn.main
