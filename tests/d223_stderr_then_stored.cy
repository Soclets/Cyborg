// Narrowings out of order are refused, with the strongest first as the fix.
// trace: D221, D223
import std;

fn.main [none] -> void:
    let $tenth: f64.stable = ~0.1;
    std::io::fmt_print.stderr.stored("{}{newln}", $tenth);
    return void;
end fn.main
