// A bound in another base, like seconds on a distance, is refused.
// trace: D260

fam.unit length:
    length.meters: base;
end fam.unit length

fam.unit time:
    time.seconds: base;
end fam.unit time

fn.main [none] returns void:
    let $d: f64.stable[unit: length.meters, min: 0.0 length.meters, max: 5.0 time.seconds] set_to 1.0 length.meters;
    return void;
end fn.main
