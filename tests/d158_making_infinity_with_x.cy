// 2 squared ten times is 2^1024, one past what an `f64` holds.
// trace: D158
import std;
import os;

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let mut $x: f64.stable = 2.0;
    for $i in range[from: 1, to: 10]:
        std::io::fmt_print(Lend.mut $out, "{}{newln}", $i);
        change $x = $x x $x;
    end for
    return void;
end fn.main
