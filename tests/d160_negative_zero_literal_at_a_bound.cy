// trace: D160, D158
import std;
import os;

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $t: f64.stable[min: 0.0, max: 1.0] = -0.0;
    std::io::fmt_print(Lend.mut $out, "fits{newln}");
    return void;
end fn.main
