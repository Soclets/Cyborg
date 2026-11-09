// trace: D203
import std;
@config(default);

fn.main [] -> void:
    decl mut $x: i32;
    assign $x = 1;
    change $x = $x + 41;
    std::io::fmt_print("{}{newln}", $x);
    return void;
end fn.main
