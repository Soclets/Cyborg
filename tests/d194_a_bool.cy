// trace: D194, D127
@config(default);
import std;

fn.main [] -> void:
    std::io::fmt_print("{}{newln}", true);
    return void;
end fn.main
