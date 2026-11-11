// A bound on a unit type writes its unit, in any unit with the same base, converted when
// compiling, so `1.0 length.kilometers` on metres is `1000.0`.
// trace: D260
import std;
import os;

fam.unit length:
    length.meters: base;
    length.kilometers set_to 1000 length.meters;
end fam.unit length

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let $near: f64.stable[unit: length.meters, min: 0.0 length.meters, max: 1.0 length.kilometers] set_to 999.5 length.meters;
    let $whole: i32[unit: length.meters, min: 0 length.meters, max: 2 length.kilometers] set_to 2000 length.meters;
    std::io::fmt_print(Lend.mut $out, "{$near, unit} {$whole, unit}{newln}");
    return void;
end fn.main
