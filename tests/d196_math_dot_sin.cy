// trace: D196, D161
import std;
@config(default);

fn.main [] -> void:
    let $x: f64 = 0.5;
    let $y: f64 = std::math.sin($x);
    return void;
end fn.main
