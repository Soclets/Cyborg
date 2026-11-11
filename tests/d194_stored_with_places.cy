// trace: D194
import std;
import os;

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $x: f64.stable = 0.5;
    std::io::fmt_print.stored(Lend.mut $out, "{places: 2}{newln}", $x);
    return void;
end fn.main
