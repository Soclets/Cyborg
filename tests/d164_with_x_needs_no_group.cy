// trace: D164, D116
import std;

fn.main [none] -> void:
    let $a: f64.stable = 8.0 div 2.0 x 3.0;
    std::io::fmt_print("{}{newln}", $a);
    return void;
end fn.main
