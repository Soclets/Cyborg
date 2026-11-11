// Points and spans in bounds weren't discussed with D260.
// trace: D260, D249

fam.unit temperature:
    temperature.kelvin: base;
    temperature.celsius = temperature.kelvin offset ~-273.15;
end fam.unit temperature

use my::temperature.celsius as celsius;

fn.main [none] returns void:
    let $now: f64.stable[unit: celsius.point, min: 0.0 celsius, max: 100.0 celsius] = 20.0 celsius;
    return void;
end fn.main
