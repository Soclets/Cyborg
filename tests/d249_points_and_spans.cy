// 2.16's example: a point plus a span is a point, a point minus a point is a span.
// trace: D249, D248, D246
import std;
import os;

fam.unit temperature:
    temperature.kelvin: base;
    temperature.celsius set_to temperature.kelvin offset ~-273.15;
end fam.unit temperature

use my::temperature.celsius as celsius;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let $now: f64.stable[unit: celsius.point] set_to 20.0 celsius;
    let $rise: f64.stable[unit: celsius.span] set_to 10.0 celsius;
    let $later: f64.stable[unit: celsius.point] set_to $now + $rise;
    let $gap: f64.stable[unit: celsius.span] set_to $later - $now;
    let $twice: f64.stable[unit: celsius.span] set_to $rise + $gap;
    std::io::fmt_print(Lend.mut $out, "{$later, unit} {$gap, unit} {$twice, unit}{newln}");
    return void;
end fn.main
