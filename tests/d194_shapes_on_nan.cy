// trace: D194, D166
import std;

fn.main [none] -> void:
    let $zero: f64.ieee = 0.0;
    std::io::fmt_print("{places: 2}{newln}", $zero div $zero);
    return void;
end fn.main
