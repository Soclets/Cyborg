// trace: D164, D159
import std;
import os;

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $zero: f64.ieee = 0.0;
    let $inf: f64.ieee = 1.0 div $zero;
    let $nan: f64.ieee = $zero div $zero;
    std::io::fmt_print(Lend.mut $out, "still running{newln}");
    let $clean: f64.stable = $nan as f64.stable;
    return void;
end fn.main
