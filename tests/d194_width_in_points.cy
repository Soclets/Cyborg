// trace: D194
@config(default);
import std;

fn.main [] -> void:
    std::io::fmt_print("{align: left, width: 8 points}{newln}", "a");
    return void;
end fn.main
