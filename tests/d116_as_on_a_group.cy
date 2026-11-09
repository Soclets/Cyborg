// trace: D116, D48, D97
@config(default);
import std;

fn.main [] -> void:
    std::io::fmt_print("{}{newln}", (7 div.trunc 2));
    std::io::fmt_print("{}{newln}", (-7 div.trunc 2));
    std::io::fmt_print("{}{newln}", (-7 div.floor 2));
    std::io::fmt_print("{}{newln}", (7 div.floor -2));
    std::io::fmt_print("{}{newln}", (8 div.exact 2));
    std::io::fmt_print("{}{newln}", (-7 mod.trunc 2));
    std::io::fmt_print("{}{newln}", (-7 mod.floor 2));
    std::io::fmt_print("{}{newln}", (7 mod.floor -2));
    return void;
end fn.main
