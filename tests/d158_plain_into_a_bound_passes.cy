// trace: D158, D20
import std;
import os;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $x: f64.stable = 20.5;
    let $t: f64.stable[min: -40.0, max: 125.0] = $x;
    std::io::fmt_print(Lend.mut $out, "fits{newln}");
    return void;
end fn.main
