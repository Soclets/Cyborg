// trace: D3
import std;

fn.main [none] -> void:
    let $s: str = "a";
    let mut $t: str = "b";
    change $t = Give $s;
    std::io::fmt_print("{}{newln}", $t);
    return void;
end fn.main
