// The language has no exponent to write, so the text is all digits, however big.
// trace: D161
import std;
@config(default);

fn.main [] -> void:
    let mut $x: f64 = 2.0;
    for $i in range[from: 1, to: 9]:
        change $x = $x x $x;
    end for
    std::io::fmt_print("{}{newln}", $x);
    return void;
end fn.main
