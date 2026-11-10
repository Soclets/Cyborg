// `as` changes a unit: a float rounds to the nearest, and an integer to a smaller unit is exact.
// trace: D255
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
    let $km: f64.length.kilometers = 2.5 length.kilometers;
    let $m: f64.length.meters = $km as f64.length.meters;
    let $whole: i32.length.kilometers = 3 length.kilometers;
    let $exact: i32.length.meters = $whole as i32.length.meters;
    let $back: f64.length.kilometers = $m as f64.length.kilometers;
    std::io::fmt_print("{$m, unit} {$exact, unit} {$back, unit}{newln}");
    return void;
end fn.main
