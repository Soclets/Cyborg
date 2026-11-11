// trace: D165, D161
import std;
import os;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let $avogadro: f64.stable set_to exponent ~6.02214076e23;
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $avogadro);
    return void;
end fn.main
