// Bounds next to a unit are written together, but a bound on a number with a unit isn't decided.
// trace: D258
@config(default);
import std;

fam.unit length:
    length.meters: base;
    length.kilometers = 1000 length.meters;
end fam.unit length

fn.main [] -> void:
    let $d: f64.stable[unit: length.meters, min: 0.0, max: 9.0] = 5.0 length.meters;
    return void;
end fn.main
