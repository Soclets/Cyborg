// trace: D166, D159
import std;

fn.main [none] -> void:
    let mut $x: f64.ieee = 2.0;
    for $i in range[from: 1, to: 10]:
        change $x = $x x $x;
    end for
    let $nan: f64.ieee = $x - $x;
    std::io::fmt_print("{}{newln}", $nan);
    std::io::fmt_print("{}{newln}", (-$nan));
    std::io::fmt_print.stored("{}{newln}", $nan);
    return void;
end fn.main
