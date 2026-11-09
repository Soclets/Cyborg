// trace: D203
@config(default);
import std;

fn.main [] -> void:
    decl $x: i32;
    std::io::fmt_print("{}{newln}", $x);
    return void;
end fn.main
