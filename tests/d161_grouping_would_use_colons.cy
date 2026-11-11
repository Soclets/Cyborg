// trace: D161, D245
import std;

fn.main [none] returns void:
    let $x: f64.stable set_to 0.5;
    let $y: f64.stable set_to std::math::sin($x);
    return void;
end fn.main
