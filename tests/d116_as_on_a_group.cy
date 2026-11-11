// trace: D116, D48, D97
import std;
import os;

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    std::io::fmt_print(Lend.mut $out, "{}{newln}", (7 div.trunc 2));
    std::io::fmt_print(Lend.mut $out, "{}{newln}", (-7 div.trunc 2));
    std::io::fmt_print(Lend.mut $out, "{}{newln}", (-7 div.floor 2));
    std::io::fmt_print(Lend.mut $out, "{}{newln}", (7 div.floor -2));
    std::io::fmt_print(Lend.mut $out, "{}{newln}", (8 div.exact 2));
    std::io::fmt_print(Lend.mut $out, "{}{newln}", (-7 mod.trunc 2));
    std::io::fmt_print(Lend.mut $out, "{}{newln}", (-7 mod.floor 2));
    std::io::fmt_print(Lend.mut $out, "{}{newln}", (7 mod.floor -2));
    return void;
end fn.main
