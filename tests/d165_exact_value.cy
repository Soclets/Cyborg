// trace: D165
import std;
@config(default);

fn.main [] -> void:
    let $a: f64 = exponent 1.5e3;
    let $b: f64 = exponent 0.0001e4;
    let $c: f64 = exponent 2.5e0;
    let $d: f64 = exponent -1.1e2;
    std::println($a as str);
    std::println($b as str);
    std::println($c as str);
    std::println($d as str);
    return void;
end fn.main
