// Nothing here says which float this is.
// trace: D161
import std;
@config(default);

fn.main [] -> void:
    std::io::fmt_print("{}{newln}", ~0.1);
    return void;
end fn.main
