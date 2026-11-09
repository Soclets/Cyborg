// trace: D194
import std;
@config(default);

fn.main [] -> void:
    std::io::fmt_print("{1, places: 2}{newln}", 1.5);
    return void;
end fn.main
