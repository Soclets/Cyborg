// Bounds next to a unit are written together, in the type's brackets, and each says its unit.
// trace: D258, D260
import std;
import os;

fam.unit length:
    length.meters: base;
    length.kilometers set_to 1000 length.meters;
end fam.unit length

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let $d: f64.stable[unit: length.meters, min: 0.0 length.meters, max: 9.0 length.meters] set_to 5.0 length.meters;
    std::io::fmt_print(Lend.mut $out, "{$d, unit}{newln}");
    return void;
end fn.main
