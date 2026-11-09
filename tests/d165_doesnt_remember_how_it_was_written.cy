// trace: D165, D162
import std;
@config(default);

fn.main [] -> void:
    let $a: f64 = exponent ~1.0e30;
    std::println.stored($a);
    return void;
end fn.main
