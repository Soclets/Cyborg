// A point converts to a point, and a span to a span.
// trace: D255, D249

fam.unit temperature:
    temperature.kelvin: base;
    temperature.celsius set_to temperature.kelvin offset ~-273.15;
end fam.unit temperature

fn.main [none] returns void:
    let $now: f64.stable[unit: temperature.celsius.point] set_to 20.0 temperature.celsius;
    let $k: f64.stable[unit: temperature.kelvin.span] set_to $now as f64.stable[unit: temperature.kelvin.span];
    return void;
end fn.main
