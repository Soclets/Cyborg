// trace: D194
@config(default);
import std;

fn.main [] -> void:
    std::io::fmt_print("{}{newln}", 1, 2);
    return void;
end fn.main
