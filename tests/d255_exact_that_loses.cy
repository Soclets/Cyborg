// `as.exact` fails if anything would be lost, the same way `div.exact` would, and that isn't decided yet.
// trace: D255
import std;

fam.unit length:
    length.meters: base;
    length.kilometers = 1000 length.meters;
    length.inches: base;
    length.yards = 36 length.inches;
    length.feet = 12 length.inches;
end fam.unit length

fn.main [none] -> void:
    let $m: i32[unit: length.meters] = 1500 length.meters;
    let $k: i32[unit: length.kilometers] = $m as.exact i32[unit: length.kilometers];
    return void;
end fn.main
