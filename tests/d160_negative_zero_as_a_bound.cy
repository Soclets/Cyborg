// `-0.0` and `0.0` stop in the same place, so these are the same type.
// trace: D160, D158
import std;
import os;

fn take [$x: f64.stable[min: 0.0, max: 1.0]] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    std::io::fmt_print(Lend.mut $out, "taken{newln}");
    return void;
end fn take

fn.main [none] returns void:
    let $t: f64.stable[min: -0.0, max: 1.0] set_to 0.5;
    take($t);
    return void;
end fn.main
