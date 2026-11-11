// A family without an offset has no points and spans.
// trace: D249
import std;

fam.unit temperature:
    temperature.kelvin: base;
    temperature.celsius set_to temperature.kelvin offset ~-273.15;
end fam.unit temperature

use my::temperature.celsius as celsius;

fam.unit length:
    length.meters: base;
end fam.unit length

fn.main [none] returns void:
    let $a: f64.stable[unit: length.meters.point] set_to 2.0 length.meters;
    return void;
end fn.main
