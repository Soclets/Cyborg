// Hello world is a source file plus a manifest.
// trace: D183, D228, D229
@config(default);
import std;

fn.main [] -> void:
    std::io::fmt_print("hi{newln}");
    return void;
end fn.main
