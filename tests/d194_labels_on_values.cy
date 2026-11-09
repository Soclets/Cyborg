// trace: D194, D114
import std;
@config(default);

fn.main [] -> void:
    std::io::fmt_print("{}{newln}", val: 1);
    return void;
end fn.main
