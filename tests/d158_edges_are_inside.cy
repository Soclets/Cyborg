// trace: D158
import std;
import os;

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $low: f64.stable[min: -40.0, max: 125.0] = -40.0;
    let $high: f64.stable[min: -40.0, max: 125.0] = 125.0;
    std::io::fmt_print(Lend.mut $out, "both{newln}");
    return void;
end fn.main
