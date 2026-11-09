// trace: D164, D116
import std;
@config(default);

fn.main [] -> void:
    let $a: f64 = 8.0 div 2.0 x 3.0;
    std::println($a as str);
    return void;
end fn.main
