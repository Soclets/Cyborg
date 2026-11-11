// Infinity minus infinity is NaN, and that's allowed here too.
// trace: D159, D158
import std;
import os;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let mut $x: f64.ieee set_to 2.0;
    for $i in range[from: 1, to: 10]:
        change $x set_to $x x $x;
    end for
    let $nan: f64.ieee set_to $x - $x;
    let $same: f64.ieee set_to $nan + 1.0;
    std::io::fmt_print(Lend.mut $out, "still running{newln}");
    return void;
end fn.main
