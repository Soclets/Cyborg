// trace: D166, D159
import std;
import os;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let mut $x: f64.ieee set_to 2.0;
    for $i in range[from: 1, to: 10]:
        change $x set_to $x x $x;
    end for
    let $nan: f64.ieee set_to $x - $x;
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $nan);
    std::io::fmt_print(Lend.mut $out, "{}{newln}", (-$nan));
    std::io::fmt_print.stored(Lend.mut $out, "{}{newln}", $nan);
    return void;
end fn.main
