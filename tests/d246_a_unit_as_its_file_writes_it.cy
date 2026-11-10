// `unit` prints a value with its unit, by its short name when `use` gives one, and by its family otherwise.
// trace: D246, D247
@config(default);
import std;

fam.unit length:
    length.meters: base;
    length.feet: base;
end fam.unit length

use my::length.feet as feet;

fn.main [] -> void:
    let $m: i32.length.meters = 5 length.meters;
    let $f: f64.feet = 2.5 feet;
    std::io::fmt_print("{$m, unit} and {$f, unit}{newln}");
    std::io::fmt_print("{unit}{newln}", $m);
    return void;
end fn.main
