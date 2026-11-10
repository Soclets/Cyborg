// Big arguments, where taking away multiples of π/2 needs π to over a thousand bits.
// trace: D256
@config(default);
import std;

use std::units::angle.radians as radians;
use std::units::angle.degrees as degrees;

fn.main [] -> void:
    let $big: f64.stable.radians = 10000000000000000000000.0 radians;
    let $huge: f64.stable.radians = exponent ~1.7976931348623157e308 radians;
    std::io::fmt_print("{} {} {}{newln}", std::math::sin($big), std::math::cos($big), std::math::tan($big));
    std::io::fmt_print("{} {} {}{newln}", std::math::sin($huge), std::math::cos($huge), std::math::tan($huge));
    return void;
end fn.main
