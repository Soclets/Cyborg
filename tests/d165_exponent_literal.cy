// trace: D165, D161
@config(default);
import std;

fn.main [] -> void:
    let $avogadro: f64.stable = exponent ~6.02214076e23;
    std::io::fmt_print("{}{newln}", $avogadro);
    return void;
end fn.main
