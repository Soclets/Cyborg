// `sin` and `tan` of zero keep its sign, and `cos` of either zero is 1.
// trace: D256
import std;
import os;

use std::units::angle.radians as radians;
use std::units::angle.degrees as degrees;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let $zero: f64.stable[unit: radians] set_to 0.0 radians;
    let $minus: f64.stable[unit: radians] set_to -0.0 radians;
    std::io::fmt_print(Lend.mut $out, "{} {} {}{newln}", std::math::sin($zero), std::math::tan($zero), std::math::cos($zero));
    std::io::fmt_print(Lend.mut $out, "{} {} {}{newln}", std::math::sin($minus), std::math::tan($minus), std::math::cos($minus));
    return void;
end fn.main
