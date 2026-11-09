// trace: D164, D116
import std;
@config(default);

fn.main [] -> void:
    let $a: f64 = 1.0 + 4.0 div 2.0;
    std::println($a as str);
    let $b: f64 = (1.0 + 4.0) div 2.0;
    std::println($b as str);
    let $c: f64 = -4.0 div 2.0;
    std::println($c as str);
    return void;
end fn.main
