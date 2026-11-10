// With radians, `sin` is right, though the oracle can't give its correctly rounded answer yet.
// trace: D245
@config(default);
import std;

use std::units::angle.radians as radians;

fn.main [] -> void:
    let $a: f64.radians = ~1.5707963267948966 radians;
    let $s: f64 = std::math::sin($a);
    return void;
end fn.main
