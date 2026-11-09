// Importing a thing doesn't shorten it, so a bare name is still nothing.
// trace: D198
import std::io::fmt_print;
@config(default);

fn.main [] -> void:
    fmt_print("hi{newln}");
    return void;
end fn.main
