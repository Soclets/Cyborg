// trace: D165, D161
import std;
import os;

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $avogadro: f64.stable = exponent ~6.02214076e23;
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $avogadro);
    return void;
end fn.main
