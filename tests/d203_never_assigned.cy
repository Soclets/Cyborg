// Never read, so nothing reads it empty.
// trace: D203
import std;
@config(default);

fn.main [] -> void:
    decl $x: i32;
    std::io::fmt_print("fine{newln}");
    return void;
end fn.main
