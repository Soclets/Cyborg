// A point plus a point is refused.
// trace: D249
import std;

fam.unit temperature:
    temperature.kelvin: base;
    temperature.celsius = temperature.kelvin offset ~-273.15;
end fam.unit temperature

use my::temperature.celsius as celsius;

fn.main [none] -> void:
    let $a: f64.stable[unit: celsius.point] = 20.0 celsius;
    let $b: f64.stable[unit: celsius.point] = 5.0 celsius;
    let $c: f64.stable[unit: celsius.point] = $a + $b;
    return void;
end fn.main
