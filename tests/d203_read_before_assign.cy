// trace: D203
import std;
@config(default);

fn.main [] -> void:
    decl $x: i32;
    std::io::fmt_print("{}{newln}", $x);
    return void;
end fn.main
