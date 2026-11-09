// trace: D194
import std;
@config(default);

fn.main [] -> void:
    let $s: str = "a";
    std::io::fmt_print("{}{newln}", Lend $s);
    return void;
end fn.main
