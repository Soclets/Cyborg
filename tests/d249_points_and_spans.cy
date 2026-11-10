// 2.16's example: a point plus a span is a point, a point minus a point is a span.
// trace: D249, D248, D246
import std;

fam.unit temperature:
    temperature.kelvin: base;
    temperature.celsius = temperature.kelvin offset ~-273.15;
end fam.unit temperature

use my::temperature.celsius as celsius;

fn.main [none] -> void:
    let $now: f64.stable[unit: celsius.point] = 20.0 celsius;
    let $rise: f64.stable[unit: celsius.span] = 10.0 celsius;
    let $later: f64.stable[unit: celsius.point] = $now + $rise;
    let $gap: f64.stable[unit: celsius.span] = $later - $now;
    let $twice: f64.stable[unit: celsius.span] = $rise + $gap;
    std::io::fmt_print("{$later, unit} {$gap, unit} {$twice, unit}{newln}");
    return void;
end fn.main
