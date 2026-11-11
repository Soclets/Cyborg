// trace: D160, D158
import std;
import os;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let $z: f64.stable set_to -0.0;
    let $t: f64.stable[min: 0.0, max: 1.0] set_to $z;
    std::io::fmt_print(Lend.mut $out, "fits{newln}");
    return void;
end fn.main
