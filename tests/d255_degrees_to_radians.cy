// Degrees convert to radians with `as`, rounding to the nearest float.
// trace: D255, D245
import std;

use std::units::angle.degrees as degrees;
use std::units::angle.radians as radians;

fn.main [none] -> void:
    let $a: f64.stable[unit: degrees] = 180.0 degrees;
    let $r: f64.stable[unit: radians] = $a as f64.stable[unit: radians];
    let $t: f64.stable[unit: std::units::angle.turns] = $a as f64.stable[unit: std::units::angle.turns];
    std::io::fmt_print("{$r, unit} {$t, unit}{newln}");
    return void;
end fn.main
