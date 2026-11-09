// trace: D194
import std;
@config(default);

fn.main [] -> void:
    let $n: i32 = -1;
    std::io::fmt_print("{base: hex}{newln}", $n);
    return void;
end fn.main
