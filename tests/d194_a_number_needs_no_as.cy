// `{}` takes a value in its readable form, so a number needs no `as str`.
// trace: D194, D95
import std;
@config(default);

fn.main [] -> void:
    let $n: i32 = 1;
    std::io::fmt_print("{}{newln}", $n);
    return void;
end fn.main
