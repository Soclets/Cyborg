// trace: D158, D20
import std;
import os;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $n: f64.stable[min: 1.0, max: 2.0] = 1.5;
    let $w: f64.stable[min: 0.0, max: 3.0] = $n;
    std::io::fmt_print(Lend.mut $out, "fits{newln}");
    return void;
end fn.main
