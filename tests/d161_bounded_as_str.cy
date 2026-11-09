// trace: D161, D158
import std;
@config(default);

fn.main [] -> void:
    let $t: f64[min: 0.0, max: 1.0] = 0.25;
    std::println($t as str);
    return void;
end fn.main
