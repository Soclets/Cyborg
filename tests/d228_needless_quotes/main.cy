// A value is quoted only when it needs to be.
// trace: D228
@config(default);
import std;

fn.main [] -> void:
    std::io::fmt_print("hi{newln}");
    return void;
end fn.main
