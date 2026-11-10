// Every program has a manifest, hello world included.
// trace: D183, D229
@config(default);
import std;

fn.main [] -> void:
    std::io::fmt_print("hi{newln}");
    return void;
end fn.main
