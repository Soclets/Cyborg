// Two units convert only if they trace back to the same base.
// trace: D255, D109
import std;

fam.unit length:
    length.meters: base;
    length.kilometers set_to 1000 length.meters;
    length.inches: base;
    length.yards set_to 36 length.inches;
    length.feet set_to 12 length.inches;
end fam.unit length

fn.main [none] returns void:
    let $m: f64.stable[unit: length.meters] set_to 1.0 length.meters;
    let $i: f64.stable[unit: length.inches] set_to $m as f64.stable[unit: length.inches];
    return void;
end fn.main
