// Whether a literal in maths is a point or a span isn't decided.
// trace: D249
import std;

fam.unit temperature:
    temperature.kelvin: base;
    temperature.celsius = temperature.kelvin offset ~-273.15;
end fam.unit temperature

use my::temperature.celsius as celsius;

fn.main [none] returns void:
    let $now: f64.stable[unit: celsius.point] = 20.0 celsius;
    let $later: f64.stable[unit: celsius.point] = $now + 10.0 celsius;
    return void;
end fn.main
