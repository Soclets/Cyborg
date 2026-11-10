// trace: D165, D162
import std;

fn.main [none] -> void:
    let $a: f64.stable = exponent ~1.0e30;
    std::io::fmt_print.stored("{}{newln}", $a);
    return void;
end fn.main
