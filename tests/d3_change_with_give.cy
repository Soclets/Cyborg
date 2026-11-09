// trace: D3
@config(default);
import std;

fn.main [] -> void:
    let $s: str = "a";
    let mut $t: str = "b";
    change $t = Give $s;
    std::io::fmt_print("{}{newln}", $t);
    return void;
end fn.main
