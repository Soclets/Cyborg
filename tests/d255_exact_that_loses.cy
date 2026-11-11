// `as.exact` fails if anything would be lost, the same way `div.exact` would, and that isn't decided yet.
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
    let $k: i32[unit: length.kilometers] set_to $m as.exact i32[unit: length.kilometers];
    return void;
end fn.main
