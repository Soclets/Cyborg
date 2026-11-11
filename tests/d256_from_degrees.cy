// Degrees converted to radians first: 90 degrees is the `f64` nearest π/2, whose sine rounds to 1.
// trace: D256, D255, D245
import std;
import os;

use std::units::angle.radians as radians;
use std::units::angle.degrees as degrees;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let $right: f64.stable[unit: degrees] set_to 90.0 degrees;
    let $half_turn: f64.stable[unit: degrees] set_to 180.0 degrees;
    std::io::fmt_print(Lend.mut $out, "{}{newln}", std::math::sin($right as f64.stable[unit: radians]));
    std::io::fmt_print(Lend.mut $out, "{} {}{newln}", std::math::sin($half_turn as f64.stable[unit: radians]), std::math::cos($half_turn as f64.stable[unit: radians]));
    return void;
end fn.main
