// A plain `as` from an integer to a bigger unit is refused.
// trace: D255
import std;

fam.unit length:
    length.meters: base;
    length.kilometers = 1000 length.meters;
    length.inches: base;
    length.yards = 36 length.inches;
    length.feet = 12 length.inches;
end fam.unit length

fn.main [none] returns void:
    let $m: i32[unit: length.meters] = 1500 length.meters;
    let $k: i32[unit: length.kilometers] = $m as i32[unit: length.kilometers];
    return void;
end fn.main
