// trace: D158, D20
import std;
import os;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let $n: f64.stable[min: 1.0, max: 2.0] set_to 1.5;
    let $w: f64.stable[min: 0.0, max: 3.0] set_to $n;
    std::io::fmt_print(Lend.mut $out, "fits{newln}");
    return void;
end fn.main
