// Infinity minus infinity is NaN, and that's allowed here too.
// trace: D159, D158
import std;

fn.main [none] -> void:
    let mut $x: f64.ieee = 2.0;
    for $i in range[from: 1, to: 10]:
        change $x = $x x $x;
    end for
    let $nan: f64.ieee = $x - $x;
    let $same: f64.ieee = $nan + 1.0;
    std::io::fmt_print("still running{newln}");
    return void;
end fn.main
