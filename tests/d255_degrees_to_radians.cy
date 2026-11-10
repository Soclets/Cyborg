// Degrees convert to radians with `as`, rounding to the nearest float.
// trace: D255, D245
@config(default);
import std;

use std::units::angle.degrees as degrees;
use std::units::angle.radians as radians;

fn.main [] -> void:
    let $a: f64.degrees = 180.0 degrees;
    let $r: f64.radians = $a as f64.radians;
    let $t: f64.std::units::angle.turns = $a as f64.std::units::angle.turns;
    std::io::fmt_print("{$r, unit} {$t, unit}{newln}");
    return void;
end fn.main
