// trace: D159, D158
import std;
import os;

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let mut $x: f64.ieee = 2.0;
    for $i in range[from: 1, to: 12]:
        change $x = $x x $x;
    end for
    std::io::fmt_print(Lend.mut $out, "still running{newln}");
    return void;
end fn.main
