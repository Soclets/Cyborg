// Degrees convert to radians with `as`, rounding to the nearest float.
// trace: D255, D245
import std;
import os;

use std::units::angle.degrees as degrees;
use std::units::angle.radians as radians;

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $a: f64.stable[unit: degrees] = 180.0 degrees;
    let $r: f64.stable[unit: radians] = $a as f64.stable[unit: radians];
    let $t: f64.stable[unit: std::units::angle.turns] = $a as f64.stable[unit: std::units::angle.turns];
    std::io::fmt_print(Lend.mut $out, "{$r, unit} {$t, unit}{newln}");
    return void;
end fn.main
