// trace: D194
@config(default);
import std;

fn.main [] -> void:
    let $s: str = "a";
    std::io::fmt_print("{}{newln}", Lend $s);
    return void;
end fn.main
