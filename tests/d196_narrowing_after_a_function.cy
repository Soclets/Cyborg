// `::` is for paths, and `.` after a function is a narrowing.
// trace: D196
import std;
@config(default);

fn.main [] -> void:
    std::io::fmt_print::stderr("x{newln}");
    return void;
end fn.main
