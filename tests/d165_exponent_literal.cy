// trace: D165, D161
import std;

fn.main [none] -> void:
    let $avogadro: f64.stable = exponent ~6.02214076e23;
    std::io::fmt_print("{}{newln}", $avogadro);
    return void;
end fn.main
