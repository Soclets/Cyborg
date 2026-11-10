// A point converts to a point, and a span to a span.
// trace: D255, D249
@config(default);

fam.unit temperature:
    temperature.kelvin: base;
    temperature.celsius = temperature.kelvin offset ~-273.15;
end fam.unit temperature

fn.main [] -> void:
    let $now: f64.temperature.celsius.point = 20.0 temperature.celsius;
    let $k: f64.temperature.kelvin.span = $now as f64.temperature.kelvin.span;
    return void;
end fn.main
