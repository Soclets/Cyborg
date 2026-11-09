// trace: D194, D166
@config(default);
import std;

fn.main [] -> void:
    let $zero: f64.ieee = 0.0;
    std::io::fmt_print("{places: 2}{newln}", $zero div $zero);
    return void;
end fn.main
