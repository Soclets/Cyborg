// The longest answer there is, and there's still no limit.
// trace: D162, D164
@config(default);
import std;

fn.main [] -> void:
    let mut $x: f64.stable = 1.0;
    for $i in range[from: 1, to: 1074]:
        change $x = $x div 2.0;
    end for
    std::io::fmt_print.stored("{}{newln}", $x);
    return void;
end fn.main
