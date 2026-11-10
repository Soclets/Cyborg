// In a family with an offset, every value says `.point` or `.span`.
// trace: D249
@config(default);
import std;

fam.unit temperature:
    temperature.kelvin: base;
    temperature.celsius = temperature.kelvin offset ~-273.15;
end fam.unit temperature

use my::temperature.celsius as celsius;

fn.main [] -> void:
    let $a: f64.stable[unit: celsius] = 20.0 celsius;
    return void;
end fn.main
