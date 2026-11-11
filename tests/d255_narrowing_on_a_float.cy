// A float rounds to the nearest, so its `as` says no narrowing.
// trace: D255, D164
import std;

fam.unit length:
    length.meters: base;
    length.kilometers set_to 1000 length.meters;
    length.inches: base;
    length.yards set_to 36 length.inches;
    length.feet set_to 12 length.inches;
end fam.unit length

fn.main [none] returns void:
    let $m: f64.stable[unit: length.meters] set_to 1500.0 length.meters;
    let $k: f64.stable[unit: length.kilometers] set_to $m as.trunc f64.stable[unit: length.kilometers];
    return void;
end fn.main
