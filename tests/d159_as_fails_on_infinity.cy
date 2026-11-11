// trace: D159
import std;
import os;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let mut $x: f64.ieee set_to 2.0;
    for $i in range[from: 1, to: 10]:
        change $x set_to $x x $x;
    end for
    std::io::fmt_print(Lend.mut $out, "before{newln}");
    let $clean: f64.stable set_to $x as f64.stable;
    std::io::fmt_print(Lend.mut $out, "after{newln}");
    return void;
end fn.main
