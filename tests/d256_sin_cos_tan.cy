// `sin`, `cos` and `tan` of radians, correctly rounded: the exact result rounded once to the nearest `f64`.
// trace: D256, D245, D161
import std;
import os;

use std::units::angle.radians as radians;
use std::units::angle.degrees as degrees;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $one: f64.stable[unit: radians] = 1.0 radians;
    let $half: f64.stable[unit: radians] = 0.5 radians;
    let $back: f64.stable[unit: radians] = -2.0 radians;
    std::io::fmt_print(Lend.mut $out, "{} {} {}{newln}", std::math::sin($one), std::math::cos($one), std::math::tan($one));
    std::io::fmt_print(Lend.mut $out, "{} {} {}{newln}", std::math::sin($half), std::math::cos($half), std::math::tan($half));
    std::io::fmt_print(Lend.mut $out, "{} {} {}{newln}", std::math::sin($back), std::math::cos($back), std::math::tan($back));
    return void;
end fn.main
