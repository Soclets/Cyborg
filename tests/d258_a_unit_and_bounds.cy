// Bounds next to a unit are written together, in the type's brackets, and each says its unit.
// trace: D258, D260
import std;

fam.unit length:
    length.meters: base;
    length.kilometers = 1000 length.meters;
end fam.unit length

fn.main [none] -> void:
    let $d: f64.stable[unit: length.meters, min: 0.0 length.meters, max: 9.0 length.meters] = 5.0 length.meters;
    std::io::fmt_print("{$d, unit}{newln}");
    return void;
end fn.main
