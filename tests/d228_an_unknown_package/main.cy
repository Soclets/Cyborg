// How a package other than `std` or `csup` is found isn't designed.
// trace: D228
import std;

fn.main [none] -> void:
    std::io::fmt_print("hi{newln}");
    return void;
end fn.main
