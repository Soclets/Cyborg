// Only NaN's comparison is open.
// trace: D160, D159
import std;
import os;

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let mut $x: f64.ieee = 2.0;
    for $i in range[from: 1, to: 10]:
        change $x = $x x $x;
    end for
    if $x > 1.5:
        std::io::fmt_print(Lend.mut $out, "above{newln}");
    end if
    return void;
end fn.main
