// trace: D161, D245
import std;

fn.main [none] returns void:
    let $x: f64.stable = 0.5;
    let $y: f64.stable = std::math::sin($x);
    return void;
end fn.main
