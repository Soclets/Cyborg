// A `need` line with no version is refused.
// trace: D228, D231
import std;

fn.main [none] -> void:
    std::io::fmt_print("hi{newln}");
    return void;
end fn.main
