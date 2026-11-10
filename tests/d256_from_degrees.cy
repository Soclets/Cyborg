// Degrees converted to radians first: 90 degrees is the `f64` nearest π/2, whose sine rounds to 1.
// trace: D256, D255, D245
@config(default);
import std;

use std::units::angle.radians as radians;
use std::units::angle.degrees as degrees;

fn.main [] -> void:
    let $right: f64.stable.degrees = 90.0 degrees;
    let $half_turn: f64.stable.degrees = 180.0 degrees;
    std::io::fmt_print("{}{newln}", std::math::sin($right as f64.stable.radians));
    std::io::fmt_print("{} {}{newln}", std::math::sin($half_turn as f64.stable.radians), std::math::cos($half_turn as f64.stable.radians));
    return void;
end fn.main
