// 0.125 is stored exactly, so two places is exactly halfway.
// trace: D194
import std;
import os;

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $x: f64.stable = 0.125;
    std::io::fmt_print(Lend.mut $out, "{places: 2}{newln}", $x);
    return void;
end fn.main
