// A point converts with its offset, and a span only scales.
// trace: D255, D249, D254
@config(default);
import std;

fam.unit temperature:
    temperature.kelvin: base;
    temperature.celsius = temperature.kelvin offset ~-273.15;
end fam.unit temperature

use my::temperature.celsius as celsius;
use my::temperature.kelvin as kelvin;

fn.main [] -> void:
    let $freezing: f64.celsius.point = 0.0 celsius;
    let $rise: f64.celsius.span = 10.0 celsius;
    let $k: f64.kelvin.point = $freezing as f64.kelvin.point;
    let $r: f64.kelvin.span = $rise as f64.kelvin.span;
    let $back: f64.celsius.point = $k as f64.celsius.point;
    std::io::fmt_print("{$k, unit} {$r, unit} {$back, unit}{newln}");
    return void;
end fn.main
