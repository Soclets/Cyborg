// `sin` is correctly rounded, which the oracle can't compute yet, so nothing can say what this prints.
// trace: D161, D196, D245
@config(default);
import std;

fn.main [] -> void:
    let $x: f64.std::units::angle.radians = 0.5 std::units::angle.radians;
    let $y: f64 = std::math::sin($x);
    return void;
end fn.main
