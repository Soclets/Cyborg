// trace: D161, D159
import std;
@config(default);

fn.main [] -> void:
    let $r: f64.ieee = 1.5;
    std::println($r as str);
    return void;
end fn.main
