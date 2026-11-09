// trace: D194
import std;
@config(default);

fn.main [] -> void:
    let $x: f64 = 0.5;
    std::io::fmt_print.stored("{places: 2}{newln}", $x);
    return void;
end fn.main
