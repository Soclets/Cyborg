// trace: D166, D159
import std;
import os;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let mut $x: f64.ieee = 2.0;
    for $i in range[from: 1, to: 10]:
        change $x = $x x $x;
    end for
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $x);
    std::io::fmt_print(Lend.mut $out, "{}{newln}", (-$x));
    std::io::fmt_print.stored(Lend.mut $out, "{}{newln}", $x);
    std::io::fmt_print.stored(Lend.mut $out, "{}{newln}", -$x);
    return void;
end fn.main
