// `with` adds features, which aren't designed.
// trace: D228
@config(default);
import std;

fn.main [] -> void:
    std::io::fmt_print("hi{newln}");
    return void;
end fn.main
