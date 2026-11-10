// Yards go to feet by way of inches, 36 per 12, which is a whole 3.
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
    let $y: i32[unit: length.yards] = 2 length.yards;
    let $f: i32[unit: length.feet] = $y as i32[unit: length.feet];
    let $i: i32[unit: length.inches] = $y as i32[unit: length.inches];
    std::io::fmt_print("{} {}{newln}", $f, $i);
    return void;
end fn.main
