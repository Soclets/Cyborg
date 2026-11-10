// `.stored` is on the whole print.
// trace: D194, D162
import std;

fn.main [none] -> void:
    let $x: f64.stable = ~0.1;
    let $y: f64.stable = 0.5;
    std::io::fmt_print.stored("{} and {}{newln}", $x, $y);
    return void;
end fn.main
