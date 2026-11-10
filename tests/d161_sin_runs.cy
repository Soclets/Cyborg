// `sin` is correctly rounded, which the oracle works out itself.
// trace: D161, D196, D245, D256
@config(default);
import std;

fn.main [] -> void:
    let $x: f64.stable[unit: std::units::angle.radians] = 0.5 std::units::angle.radians;
    let $y: f64.stable = std::math::sin($x);
    return void;
end fn.main
