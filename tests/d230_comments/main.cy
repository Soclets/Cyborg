// Comments in a manifest are `//`, on their own line or trailing.
// trace: D230
@config(default);
import std;

fn.main [] -> void:
    std::io::fmt_print("hi{newln}");
    return void;
end fn.main
