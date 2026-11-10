// Two units convert only if they trace back to the same base.
// trace: D255, D109
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
    let $m: f64.stable.length.meters = 1.0 length.meters;
    let $i: f64.stable.length.inches = $m as f64.stable.length.inches;
    return void;
end fn.main
