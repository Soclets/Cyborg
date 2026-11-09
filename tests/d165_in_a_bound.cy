// trace: D165, D158
import std;
@config(default);

fn.main [] -> void:
    let $x: f64[min: 0.0, max: exponent 1.0e6] = exponent 2.5e5;
    std::println($x as str);
    return void;
end fn.main
