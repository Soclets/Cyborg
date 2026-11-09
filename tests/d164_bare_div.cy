// trace: D164
import std;
@config(default);

fn.main [] -> void:
    let $a: f64 = 7.0 div 2.0;
    std::println($a as str);
    return void;
end fn.main
