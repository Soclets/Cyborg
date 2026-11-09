// trace: D194
import std;
@config(default);

fn.main [] -> void:
    std::io::fmt_print("{2}{newln}", 1, 2);
    return void;
end fn.main
