// trace: D194
@config(default);
import std;

fn.main [] -> void:
    std::io::fmt_print("{2} {1} {2}{newln}", "a", "b");
    return void;
end fn.main
