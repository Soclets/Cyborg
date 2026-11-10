// A float rounds to the nearest, so its `as` says no narrowing.
// trace: D255, D164
@config(default);
import std;

fam.unit length:
    length.meters: base;
    length.kilometers = 1000 length.meters;
    length.inches: base;
    length.yards = 36 length.inches;
    length.feet = 12 length.inches;
end fam.unit length

fn.main [] -> void:
    let $m: f64.length.meters = 1500.0 length.meters;
    let $k: f64.length.kilometers = $m as.trunc f64.length.kilometers;
    return void;
end fn.main
