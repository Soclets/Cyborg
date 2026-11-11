// `as` changes a unit: a float rounds to the nearest, and an integer to a smaller unit is exact.
// trace: D255
import std;
import os;

fam.unit length:
    length.meters: base;
    length.kilometers set_to 1000 length.meters;
    length.inches: base;
    length.yards set_to 36 length.inches;
    length.feet set_to 12 length.inches;
end fam.unit length

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let $km: f64.stable[unit: length.kilometers] set_to 2.5 length.kilometers;
    let $m: f64.stable[unit: length.meters] set_to $km as f64.stable[unit: length.meters];
    let $whole: i32[unit: length.kilometers] set_to 3 length.kilometers;
    let $exact: i32[unit: length.meters] set_to $whole as i32[unit: length.meters];
    let $back: f64.stable[unit: length.kilometers] set_to $m as f64.stable[unit: length.kilometers];
    std::io::fmt_print(Lend.mut $out, "{$m, unit} {$exact, unit} {$back, unit}{newln}");
    return void;
end fn.main
