// Nothing says whether a manifest can leave out a line its examples all have.
// trace: D183, D228
@config(default);
import std;

fn.main [] -> void:
    std::io::fmt_print("hi{newln}");
    return void;
end fn.main
