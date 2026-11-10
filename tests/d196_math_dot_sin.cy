// trace: D196, D161
@config(default);
import std;

fn.main [] -> void:
    let $x: f64.stable = 0.5;
    let $y: f64.stable = std::math.sin($x);
    return void;
end fn.main
