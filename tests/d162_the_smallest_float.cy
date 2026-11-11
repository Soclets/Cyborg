// The longest answer there is, and there's still no limit.
// trace: D162, D164
import std;
import os;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let mut $x: f64.stable = 1.0;
    for $i in range[from: 1, to: 1074]:
        change $x = $x div 2.0;
    end for
    std::io::fmt_print.stored(Lend.mut $out, "{}{newln}", $x);
    return void;
end fn.main
