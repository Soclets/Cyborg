// trace: D167, D166
import std;
import os;

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let mut $x: f64.ieee = 2.0;
    for $i in range[from: 1, to: 10]:
        change $x = $x x $x;
    end for
    let $nan: f64.ieee = $x - $x;
    let $more: f64.ieee = $nan x 0.0 + 1.0;
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $more);
    return void;
end fn.main
