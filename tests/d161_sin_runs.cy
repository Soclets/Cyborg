// `sin` is correctly rounded, which the oracle works out itself.
// trace: D161, D196, D245, D256
import std;

fn.main [none] -> void:
    let $x: f64.stable[unit: std::units::angle.radians] = 0.5 std::units::angle.radians;
    let $y: f64.stable = std::math::sin($x);
    return void;
end fn.main
