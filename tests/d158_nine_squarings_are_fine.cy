// trace: D158
import std;
import os;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let mut $x: f64.stable set_to 2.0;
    for $i in range[from: 1, to: 9]:
        change $x set_to $x x $x;
    end for
    std::io::fmt_print(Lend.mut $out, "2^512 is fine{newln}");
    return void;
end fn.main
