// A point plus a point is refused.
// trace: D249
@config(default);
import std;

fam.unit temperature:
    temperature.kelvin: base;
    temperature.celsius = temperature.kelvin offset ~-273.15;
end fam.unit temperature

use my::temperature.celsius as celsius;

fn.main [] -> void:
    let $a: f64.celsius.point = 20.0 celsius;
    let $b: f64.celsius.point = 5.0 celsius;
    let $c: f64.celsius.point = $a + $b;
    return void;
end fn.main
