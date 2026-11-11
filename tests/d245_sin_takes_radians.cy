// With radians, `sin` runs.
// trace: D245, D256
import std;

use std::units::angle.radians as radians;

fn.main [none] returns void:
    let $a: f64.stable[unit: radians] set_to ~1.5707963267948966 radians;
    let $s: f64.stable set_to std::math::sin($a);
    return void;
end fn.main
