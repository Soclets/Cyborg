// A `need` line with no version is refused.
// trace: D228, D231
@config(default);
import std;

fn.main [] -> void:
    std::io::fmt_print("hi{newln}");
    return void;
end fn.main
