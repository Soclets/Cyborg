// Published worst cases: sin of the `f64` nearest π/2 rounds up to 1, and cos(2^-26) just misses it.
// trace: D256
import std;
import os;

use std::units::angle.radians as radians;
use std::units::angle.degrees as degrees;

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $near: f64.stable[unit: radians] = ~1.5707963267948966 radians;
    let $tiny: f64.stable[unit: radians] = 0.0000000149011611938476562500 radians;
    std::io::fmt_print(Lend.mut $out, "{} {}{newln}", std::math::sin($near), std::math::cos($tiny));
    std::io::fmt_print.stored(Lend.mut $out, "{}{newln}", std::math::cos($tiny));
    return void;
end fn.main
