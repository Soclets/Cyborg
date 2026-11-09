// trace: D132, D7
@config(default);
import std;

fn.main [] -> void:
    let $n: i32 = 5;
    let $s: str = $n as str ?? "none";
    std::io::fmt_print("{}{newln}", $s);
    return void;
end fn.main
