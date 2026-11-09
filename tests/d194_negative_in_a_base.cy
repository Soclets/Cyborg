// trace: D194
@config(default);
import std;

fn.main [] -> void:
    let $n: i32 = -1;
    std::io::fmt_print("{base: hex}{newln}", $n);
    return void;
end fn.main
