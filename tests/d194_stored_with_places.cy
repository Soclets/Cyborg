// trace: D194
import std;

fn.main [none] -> void:
    let $x: f64.stable = 0.5;
    std::io::fmt_print.stored("{places: 2}{newln}", $x);
    return void;
end fn.main
