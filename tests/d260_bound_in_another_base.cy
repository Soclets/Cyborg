// A bound in another base, like seconds on a distance, is refused.
// trace: D260
@config(default);

fam.unit length:
    length.meters: base;
end fam.unit length

fam.unit time:
    time.seconds: base;
end fam.unit time

fn.main [] -> void:
    let $d: f64.stable[unit: length.meters, min: 0.0 length.meters, max: 5.0 time.seconds] = 1.0 length.meters;
    return void;
end fn.main
