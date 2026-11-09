// 2 squared ten times is 2^1024, one past what an `f64` holds.
// trace: D158
@config(default);
import std;

fn.main [] -> void:
    let mut $x: f64 = 2.0;
    for $i in range[from: 1, to: 10]:
        std::io::fmt_print("{}{newln}", $i);
        change $x = $x x $x;
    end for
    return void;
end fn.main
