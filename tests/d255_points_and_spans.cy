// A point converts with its offset, and a span only scales.
// trace: D255, D249, D254
import std;
import os;

fam.unit temperature:
    temperature.kelvin: base;
    temperature.celsius set_to temperature.kelvin offset ~-273.15;
end fam.unit temperature

use my::temperature.celsius as celsius;
use my::temperature.kelvin as kelvin;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let $freezing: f64.stable[unit: celsius.point] set_to 0.0 celsius;
    let $rise: f64.stable[unit: celsius.span] set_to 10.0 celsius;
    let $k: f64.stable[unit: kelvin.point] set_to $freezing as f64.stable[unit: kelvin.point];
    let $r: f64.stable[unit: kelvin.span] set_to $rise as f64.stable[unit: kelvin.span];
    let $back: f64.stable[unit: celsius.point] set_to $k as f64.stable[unit: celsius.point];
    std::io::fmt_print(Lend.mut $out, "{$k, unit} {$r, unit} {$back, unit}{newln}");
    return void;
end fn.main
