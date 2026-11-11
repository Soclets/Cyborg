// A plain `as` from an integer to a bigger unit is refused.
// trace: D255
import std;

fam.unit length:
    length.meters: base;
    length.kilometers set_to 1000 length.meters;
    length.inches: base;
    length.yards set_to 36 length.inches;
    length.feet set_to 12 length.inches;
end fam.unit length

fn.main [none] returns void:
    let $m: i32[unit: length.meters] set_to 1500 length.meters;
    let $k: i32[unit: length.kilometers] set_to $m as i32[unit: length.kilometers];
    return void;
end fn.main
