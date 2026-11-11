// The language has no exponent to write, so the text is all digits, however big.
// trace: D161
import std;
import os;

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let mut $x: f64.stable = 2.0;
    for $i in range[from: 1, to: 9]:
        change $x = $x x $x;
    end for
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $x);
    return void;
end fn.main
