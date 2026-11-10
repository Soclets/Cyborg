// trace: D161, D158
import std;

fn.main [none] -> void:
    let $t: f64.stable[min: 0.0, max: 1.0] = 0.25;
    std::io::fmt_print("{}{newln}", $t);
    return void;
end fn.main
