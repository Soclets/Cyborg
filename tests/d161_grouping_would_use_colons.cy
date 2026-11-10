// trace: D161, D245
@config(default);
import std;

fn.main [] -> void:
    let $x: f64 = 0.5;
    let $y: f64 = std::math::sin($x);
    return void;
end fn.main
