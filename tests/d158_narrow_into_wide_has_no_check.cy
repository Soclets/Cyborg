// trace: D158, D20
import std;
@config(default);

fn.main [] -> void:
    let $n: f64[min: 1.0, max: 2.0] = 1.5;
    let $w: f64[min: 0.0, max: 3.0] = $n;
    std::println("fits");
    return void;
end fn.main
