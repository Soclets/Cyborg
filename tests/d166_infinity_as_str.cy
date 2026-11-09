// trace: D166, D159
@config(default);
import std;

fn.main [] -> void:
    let mut $x: f64.ieee = 2.0;
    for $i in range[from: 1, to: 10]:
        change $x = $x x $x;
    end for
    std::io::fmt_print("{}{newln}", $x);
    std::io::fmt_print("{}{newln}", (-$x));
    std::io::fmt_print.stored("{}{newln}", $x);
    std::io::fmt_print.stored("{}{newln}", -$x);
    return void;
end fn.main
