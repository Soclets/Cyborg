// trace: D194
import std;
@config(default);

fn.main [] -> void:
    std::io::fmt_print("{align: left, width: 8 points}{newln}", "a");
    return void;
end fn.main
