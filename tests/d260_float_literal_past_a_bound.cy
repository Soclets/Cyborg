// A float literal past a bound written in another unit is refused when compiling too.
// trace: D260, D158
@config(default);

fam.unit length:
    length.meters: base;
    length.kilometers = 1000 length.meters;
end fam.unit length

fn.main [] -> void:
    let $far: f64.stable[unit: length.meters, min: 0.0 length.meters, max: 0.5 length.kilometers] = 500.5 length.meters;
    return void;
end fn.main
