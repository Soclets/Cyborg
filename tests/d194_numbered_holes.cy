// trace: D194
import std;
@config(default);

fn.main [] -> void:
    std::io::fmt_print("{2} {1} {2}{newln}", "a", "b");
    return void;
end fn.main
