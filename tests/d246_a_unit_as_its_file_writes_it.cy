// `unit` prints a value with its unit, by its short name when `use` gives one, and by its family otherwise.
// trace: D246, D247
import std;
import os;

fam.unit length:
    length.meters: base;
    length.feet: base;
end fam.unit length

use my::length.feet as feet;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $m: i32[unit: length.meters] = 5 length.meters;
    let $f: f64.stable[unit: feet] = 2.5 feet;
    std::io::fmt_print(Lend.mut $out, "{$m, unit} and {$f, unit}{newln}");
    std::io::fmt_print(Lend.mut $out, "{unit}{newln}", $m);
    return void;
end fn.main
