// A comma is followed by one space.
// trace: D213
@config(default);
import std;

fn.main [] -> void:
    std::io::fmt_print("{} {}{newln}", 1,2);
    return void;
end fn.main
