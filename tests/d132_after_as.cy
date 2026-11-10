// trace: D132, D7
import std;

fn.main [none] -> void:
    let $n: i32 = 5;
    let $s: str = $n as str ?? "none";
    std::io::fmt_print("{}{newln}", $s);
    return void;
end fn.main
