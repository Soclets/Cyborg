// trace: D194, D114
@config(default);
import std;

fn.main [] -> void:
    std::io::fmt_print("{}{newln}", val: 1);
    return void;
end fn.main
